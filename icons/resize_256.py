import os
from PIL import Image

src_path = r"E:\マイドライブ\dev\vlt-syslog\vlt-syslogd\icons_backup_20260129_1915\vlt_syslogd_icon.png"
dst_path = r"E:\マイドライブ\dev\vlt-syslog\vlt-syslogd\icons\vlt_syslogd_icon_256px.png"

print(f"Opening: {src_path}")

if not os.path.exists(src_path):
    print(f"Error: Source file not found at {src_path}")
    exit(1)

with Image.open(src_path) as img:
    print(f"Original size: {img.size}")
    
    # Resize to 256x256 using High Quality resampling (LANCZOS)
    img_resized = img.resize((256, 256), Image.Resampling.LANCZOS)
    
    print(f"Resized to: {img_resized.size}")
    img_resized.save(dst_path)
    print(f"Saved to: {dst_path}")
