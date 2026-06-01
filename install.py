import os
import platform
import sys

#NOTE Написано в 2 часа ночи. 
#COLORES FOR OUTPUT

GREEN = "\033[92m";
RED = "\033[91m";
YELLOW = "\033[93m";
RESET = "\033[0m";

def print_step(msg):
	print(f"{GREEN}==>{RESET} {msg}");

def print_error(msg):
	print(f"{RED}Error:{RESET} {msg}");

def print_warning(msg):
	print(f"{YELLOW}Warning:{RESET} {msg}");

def check_for_heresy():
	if platform.system() == "Windows" or "microsoft" in platform.release().lower():
		for _ in range(100):
			print(f"{RED}WINDOWS DETECTED! INSTALL LINUX!{RESET}");

		print(f"{RED}FATAL ERROR: UNACCEPTABLE ENVIRONMENT DETECTED!{RESET}");
		print("--------------------------------------------------");
		print("Your system is infested with backslashes and bloated registry.");
		print("calc refuses to coexist with Explorer.exe or PowerShell.");
		print("");
		print("ACTION REQUIRED:");
		print("1. Wipe your drive.");
		print("2. Install Arch Linux.");
		print("3. Come back when you have a real /usr/bin.");
		print("--------------------------------------------------");
		
		# ASCII ARCH LOGO - МОНУМЕНТ ИСТИНЫ
		print(f"{GREEN}");
		print("       /\\");
		print("      /  \\");
		print("     /    \\");
		print("    /      \\");
		print("   /   _    \\");
		print("  /   ( )    \\");
		print(" /            \\");
		print("/_/\________/\_\\");
		print(f"{RESET}");
		
		print(f"{YELLOW}Don't disgrace your computer, install ARCH!{RESET}");
		print("EMERGENCY EXIT INITIATED...");
		
		os.system("start https://archlinux.org"); #sos.system
		print("Opening your salvation in Edge... Goodbye.");
		sys.exit(10);

check_for_heresy();

print_step("Start building project");
os.system("cargo build --release");

user_input = input("Copy binary to /usr/bin/calc? (y/N): ").strip().lower();
if user_input in ('y', 'yes'):
	os.system("sudo cp target/release/Simple_calc /usr/bin/calc");
	print_step("Binary installed to /usr/bin/arrowc");
else:
	user_bin = os.path.expanduser("~/.local/bin");
	os.makedirs(user_bin, exist_ok=True);
	os.system(f"cp target/release/Simple_calc {user_bin}/calc");
	print_step(f"Binary installed to {user_bin}/calc PATH needed");

