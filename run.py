import os
import pathlib
import subprocess

outfile = pathlib.Path("a.out")
env = {**os.environ, "RUST_BACKTRACE": "1"}

if outfile.is_file():
    outfile.unlink()

result = subprocess.check_call(["cargo", "run"], env=env)
if result != 0:
    print("Failed to compile code")
    exit(result)

subprocess.check_call(["cc", outfile])
subprocess.check_call(["otool", "-tvVBd", outfile])
