#Intended to be used on "deduped" folder from F8 3dmigoto
#example filename: 0d5011b0-BC1_UNORM_SRGB.jpg
#result: 0d5011b0.jpg
import os

# Folder path where your files are
folder_path = r"please_fill_in_correct_path"

for root, dirs, files in os.walk(folder_path):
    for filename in files:
        try:
            # Get extension
            _, ext = os.path.splitext(filename)

            # Extract the first 8 characters
            new_name_part = filename[0 : 8]

            if len(new_name_part) < 8:
                print(f"Skipping (name too short): {filename}")
                continue

            new_filename = new_name_part + ext

            # Full paths
            old_file = os.path.join(root, filename)
            new_file = os.path.join(root, new_filename)

            # Check for name conflict
            if os.path.exists(new_file):
                print(f"Skipping (target exists): {new_filename}")
                continue

            os.rename(old_file, new_file)
            print(f"Renamed: {old_file} → {new_file}")

        except Exception as e:
            print(f"Error processing {filename}: {e}")
