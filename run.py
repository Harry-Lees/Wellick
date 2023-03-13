import argparse
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


compile_stdlib = True

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("file", help="The file to compile")
    args = parser.parse_args()

    outfile = pathlib.Path("a.out")
    env = {**os.environ, "RUST_BACKTRACE": "1"}

    if outfile.is_file():
        outfile.unlink()

    result = subprocess.run(["cargo", "run", "--release", args.file], env=env)
    if result.returncode != 0:
        cprint("Failed to compile", RED)
        exit(result.returncode)

    if platform.system() == "Darwin":
        subprocess.check_call(["cc", outfile])
        result = subprocess.run(["./a.out"])
        cprint("Successfully compiled for MacOS", GREEN)
        print(result)

    if platform.system() == "Windows":
        if compile_stdlib:
            cprint("Compiling stdlib", GREEN)
            subprocess.check_call(["cl", "builtins.c", "/LD", "/MD", "/Wall"])
        subprocess.check_call(["LINK", outfile, "builtins", "/SUBSYSTEM:CONSOLE"])
        result = subprocess.run(["a.exe"])
        cprint("Successfully compiled for Windows", GREEN)
        print(result)
