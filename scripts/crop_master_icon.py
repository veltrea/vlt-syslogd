from PIL import Image
import sys
import os

def crop_center_expand(image_path, crop_percent=0.15):
    """
    画像の周囲(crop_percent)を切り落とし、
    中央部分を元のサイズに拡大して上書き保存する。
    crop_percent=0.15 なら、上下左右15%ずつカットする（中央70%が残る）。
    """
    try:
        img = Image.open(image_path).convert("RGBA")
        width, height = img.size
        
        # クロップ範囲の計算
        left = width * crop_percent
        top = height * crop_percent
        right = width * (1 - crop_percent)
        bottom = height * (1 - crop_percent)
        
        print(f"Original: {width}x{height}")
        print(f"Cropping to: {left:.1f}, {top:.1f}, {right:.1f}, {bottom:.1f}")
        
        # クロップ
        cropped = img.crop((left, top, right, bottom))
        
        # リサイズ（高品質）
        resized = cropped.resize((width, height), Image.Resampling.LANCZOS)
        
        # バックアップ作成
        backup_path = image_path + ".bak"
        if not os.path.exists(backup_path):
            img.save(backup_path)
            print(f"Backup saved to: {backup_path}")
        
        # 上書き保存
        resized.save(image_path)
        print(f"Successfully processed: {image_path}")
        
    except Exception as e:
        print(f"Error processing {image_path}: {e}")

if __name__ == "__main__":
    target_file = r"e:\マイドライブ\dev\vlt-syslog\vlt-syslogd\icons\vlt_syslogd_icon.png"
    # 少し大胆にカットする（枠が邪魔とのことなので20%前後カットしても良いかもしれないが、まずは15%で）
    crop_center_expand(target_file, crop_percent=0.15)
