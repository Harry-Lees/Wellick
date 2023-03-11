import os
import pathlib
import platform
import subprocess

BLACK = "\u001b[30m"
RED = "\u001b[31m"
GREEN = "\u001b[32m"
WHITE = "\u001b[97m"


def cprint(text, color):
    print(f"{color}{text}{WHITE}")


if __name__ == "__main__":
    outfile = pathlib.Path("a.out")
    env = {**os.environ, "RUST_BACKTRACE": "1"}

    if outfile.is_file():
        outfile.unlink()

    result = subprocess.run(["cargo", "run", "--release"], env=env)
    if result.returncode != 0:
        cprint("Failed to compile", RED)
        exit(result.returncode)

    if platform.system() == "Darwin":
        subprocess.check_call(["cc", outfile])
        result = subprocess.run(["./a.out"])
        cprint("Successfully compiled for MacOS", GREEN)
        print(result)

    if platform.system() == "Windows":
        subprocess.check_call(["LINK", outfile, "/ENTRY:main"])
        result = subprocess.run(["a.exe"])
        cprint("Successfully compiled for Windows", GREEN)
        print(result)
