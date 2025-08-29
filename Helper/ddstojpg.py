import os
import subprocess

def convert_dds_to_jpg(input_folder, texconv_path):
    """
    Convert all DDS files in the input folder to JPG format using texconv.exe
    
    Args:
        input_folder (str): Path to the folder containing DDS files
        texconv_path (str): Path to texconv.exe
    """
    # Ensure the input folder exists
    if not os.path.exists(input_folder):
        print(f"Error: Folder '{input_folder}' does not exist.")
        return
    
    # Ensure texconv.exe exists
    if not os.path.exists(texconv_path):
        print(f"Error: texconv.exe not found at '{texconv_path}'.")
        return
    
    # Find all .dds files in the input folder and its subfolders
    for root, dirs, files in os.walk(input_folder):
        for file in files:
            if file.lower().endswith('.dds'):
                dds_path = os.path.join(root, file)
                output_path = os.path.join(root, os.path.splitext(file)[0] + '.jpg')
                
                # Build the texconv command
                command = [
                    texconv_path,
                    '-ft', 'jpg',       # Output format
                    '-y',               # Overwrite without prompt
                    '-o', root,         # Output directory (same as input)
                    '-f', 'R8G8B8A8_UNORM',  # Format (if needed)
                    dds_path            # Input file
                ]
                
                print(f"Converting: {dds_path} to {output_path}")
                
                try:
                    # Run texconv
                    subprocess.run(command, check=True)
                    print(f"Successfully converted {file}")
                except subprocess.CalledProcessError as e:
                    print(f"Failed to convert {file}: {e}")

if __name__ == "__main__":
    # Configuration - modify these paths
    input_folder = r"please_fill_in_correct_path"  # Folder containing DDS files
    texconv_path = r"from_microsoft-texconv.exe"     # Path to texconv.exe
    
    convert_dds_to_jpg(input_folder, texconv_path)
    print("Conversion process completed.")