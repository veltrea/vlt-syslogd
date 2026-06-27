import ctypes
import struct
import os
import shutil
from ctypes import wintypes

# Define Windows API constants and types
kernel32 = ctypes.windll.kernel32

RT_ICON = 3
RT_GROUP_ICON = 14

# Structures for ICO file parsing and Resource Group Icon format
# https://docs.microsoft.com/en-us/windows/win32/menurc/resource-file-formats

class ICONDIR(ctypes.Structure):
    _pack_ = 1
    _fields_ = [
        ('idReserved', wintypes.WORD),
        ('idType', wintypes.WORD),
        ('idCount', wintypes.WORD),
    ]

class ICONDIRENTRY(ctypes.Structure):
    _pack_ = 1
    _fields_ = [
        ('bWidth', wintypes.BYTE),
        ('bHeight', wintypes.BYTE),
        ('bColorCount', wintypes.BYTE),
        ('bReserved', wintypes.BYTE),
        ('wPlanes', wintypes.WORD),
        ('wBitCount', wintypes.WORD),
        ('dwBytesInRes', wintypes.DWORD),
        ('dwImageOffset', wintypes.DWORD),
    ]

# GRPICONDIRENTRY is slightly different from ICONDIRENTRY
# It replaces dwImageOffset with nID (the resource ID of the icon)
class GRPICONDIRENTRY(ctypes.Structure):
    _pack_ = 1
    _fields_ = [
        ('bWidth', wintypes.BYTE),
        ('bHeight', wintypes.BYTE),
        ('bColorCount', wintypes.BYTE),
        ('bReserved', wintypes.BYTE),
        ('wPlanes', wintypes.WORD),
        ('wBitCount', wintypes.WORD),
        ('dwBytesInRes', wintypes.DWORD),
        ('nID', wintypes.WORD),
    ]

def embed_icon(exe_path, ico_path):
    print(f"Target EXE: {exe_path}")
    print(f"Source ICO: {ico_path}")

    if not os.path.exists(exe_path):
        print("Error: Executable not found.")
        return False
    if not os.path.exists(ico_path):
        print("Error: Icon file not found.")
        return False

    # Backup the executable
    backup_path = exe_path + ".bak"
    shutil.copy2(exe_path, backup_path)
    print(f"Created backup: {backup_path}")

    # Read the ICO file
    with open(ico_path, 'rb') as f:
        ico_data = f.read()

    # Parse ICO header
    icon_dir = ICONDIR.from_buffer_copy(ico_data)
    if icon_dir.idReserved != 0 or icon_dir.idType != 1:
        print("Error: Invalid ICO file.")
        return False

    count = icon_dir.idCount
    print(f"Found {count} icons in ICO file.")

    entries = []
    images = []
    
    offset = ctypes.sizeof(ICONDIR)
    
    # Read all icon entries and data
    for i in range(count):
        entry = ICONDIRENTRY.from_buffer_copy(ico_data, offset)
        entries.append(entry)
        
        # Extract image data
        f_offset = entry.dwImageOffset
        size = entry.dwBytesInRes
        images.append(ico_data[f_offset : f_offset + size])
        
        offset += ctypes.sizeof(ICONDIRENTRY)

    # Define argtypes for UpdateResourceW to ensure correct parameter passing
    kernel32.UpdateResourceW.argtypes = [
        wintypes.HANDLE,    # hUpdate
        wintypes.LPCWSTR,   # lpType
        wintypes.LPCWSTR,   # lpName
        wintypes.WORD,      # wLanguage
        ctypes.c_void_p,    # lpData
        wintypes.DWORD      # cb
    ]

    # Helper to handle resource IDs (MakeIntResource)
    def MAKEINTRESOURCE(i):
        return ctypes.cast(i, wintypes.LPCWSTR)

    # Begin Update
    hUpdate = kernel32.BeginUpdateResourceW(exe_path, False)
    if not hUpdate:
        print(f"Error: BeginUpdateResource failed. Error code: {kernel32.GetLastError()}")
        return False

    # Update RT_ICON resources
    # We will use IDs starting from 1
    base_id = 1
    
    try:
        # Prepare GRPICONDIR structure
        grp_icon_dir = ICONDIR()
        grp_icon_dir.idReserved = 0
        grp_icon_dir.idType = 1
        grp_icon_dir.idCount = count
        
        grp_data = bytearray(grp_icon_dir)

        for i in range(count):
            ico_entry = entries[i]
            img_data = images[i]
            icon_id = base_id + i
            
            print(f"Updating Icon ID {icon_id}: Size={len(img_data)} bytes, Width={ico_entry.bWidth}, Height={ico_entry.bHeight}")

            # Use create_string_buffer to ensure binary data is passed correctly as a pointer
            data_buffer = ctypes.create_string_buffer(img_data, len(img_data))

            # Write the individual icon resource (RT_ICON)
            # RT_ICON is 3. We cast it to LPCWSTR (which works as MAKEINTRESOURCE(3))
            ret = kernel32.UpdateResourceW(
                hUpdate,
                MAKEINTRESOURCE(RT_ICON),
                MAKEINTRESOURCE(icon_id),
                wintypes.WORD(0), # Language: Neutral
                ctypes.cast(data_buffer, ctypes.c_void_p),
                len(img_data)
            )
            
            if not ret:
                err = kernel32.GetLastError()
                print(f"UpdateResource (RT_ICON) failed for ID {icon_id}. Error: {err}")
                kernel32.EndUpdateResourceW(hUpdate, True)
                return False

            # Create GRPICONDIRENTRY
            grp_entry = GRPICONDIRENTRY()
            grp_entry.bWidth = ico_entry.bWidth
            grp_entry.bHeight = ico_entry.bHeight
            grp_entry.bColorCount = ico_entry.bColorCount
            grp_entry.bReserved = ico_entry.bReserved
            grp_entry.wPlanes = ico_entry.wPlanes
            grp_entry.wBitCount = ico_entry.wBitCount
            grp_entry.dwBytesInRes = ico_entry.dwBytesInRes
            grp_entry.nID = icon_id
            
            grp_data.extend(bytearray(grp_entry))
        
        # Write the group icon resource (RT_GROUP_ICON)
        # ID 1 for the main icon group.
        MAIN_ICON_ID = 1
        print(f"Updating Group Icon ID {MAIN_ICON_ID}: Size={len(grp_data)} bytes")
        
        grp_data_buffer = ctypes.create_string_buffer(bytes(grp_data), len(grp_data))
        
        ret = kernel32.UpdateResourceW(
            hUpdate,
            MAKEINTRESOURCE(RT_GROUP_ICON),
            MAKEINTRESOURCE(MAIN_ICON_ID),
            wintypes.WORD(0),
            ctypes.cast(grp_data_buffer, ctypes.c_void_p),
            len(grp_data)
        )
        
        if not ret:
             err = kernel32.GetLastError()
             print(f"UpdateResource (RT_GROUP_ICON) failed. Error: {err}")
             kernel32.EndUpdateResourceW(hUpdate, True)
             return False

        # Commit changes
        if not kernel32.EndUpdateResourceW(hUpdate, False):
             err = kernel32.GetLastError()
             print(f"EndUpdateResource failed. Error: {err}")
             return False
             
        print("Successfully embedded icon resources.")
        return True

    except Exception as e:
        print(f"Exception occurred: {e}")
        # Discard changes if something went wrong
        kernel32.EndUpdateResourceW(hUpdate, True)
        return False

if __name__ == "__main__":
    # Hardcoded paths based on user request
    # E:\マイドライブ\dev\vlt-syslog\vlt-syslogd\icons\vlt-syslogd.ico
    # E:\マイドライブ\dev\vlt-syslog\vlt-syslogd\target\release\vlt-syslogd.exe
    
    base_dir = r"E:\マイドライブ\dev\vlt-syslog\vlt-syslogd"
    icon_file = os.path.join(base_dir, "icons", "vlt-syslogd.ico")
    exe_file = os.path.join(base_dir, "target", "release", "vlt-syslogd.exe")

    embed_icon(exe_file, icon_file)
