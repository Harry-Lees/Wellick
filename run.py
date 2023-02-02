import os
import pathlib
import subprocess

BLACK = "\u001b[30m"
RED = "\u001b[31m"
GREEN = "\u001b[32m"
WHITE = "\u001b[97m"

def cprint(text, color):
    print(f"{color}{text}{WHITE}")

outfile = pathlib.Path("a.out")
env = {**os.environ, "RUST_BACKTRACE": "1"}

if outfile.is_file():
    outfile.unlink()

result = subprocess.run(["cargo", "run"], env=env)
if result.returncode != 0:
    cprint("Failed to compile", RED)
    exit(result.returncode)

subprocess.check_call(["cc", outfile])
subprocess.check_call(["otool", "-tvVBd", outfile])
cprint("Successfully compiled", GREEN)

result = subprocess.run(["./a.out"])
print(result)