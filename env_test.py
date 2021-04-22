import os

print('>>>>>>>>>> PRE-BENCHMARK CHECKS')

has_error = False

if 'CABEAN_BIN' in os.environ:	
	print("CABEAN path:", os.environ['CABEAN_BIN'])
	code = os.system(os.environ['CABEAN_BIN'] + ' &> /dev/null')
	if code == 256 or code == 0: # 0 on linux, 256 on macos
		print("CABEAN executable ok.")
	else:
		has_error = True
		print("!!!ERROR!!! CABEAN executable exit code", code)
else:
	print("CABEAN path: UNDEFINED")

timeout = 'none'

if timeout == 'none':
	code = os.system('timeout --help &> /dev/null')
	if code == 0:
		timeout = 'timeout'
		print("Timeout utility ok.")

if timeout == 'none':
	code = os.system('gtimeout --help &> /dev/null')
	if code == 0:
		timeout = 'gtimeout'
		print("Timeout utility ok.")

if timeout == 'none': 
	has_error = True
	print('!!!ERROR!!! No timeout utility found.')

cargo_ok = os.system('cargo --version &> /dev/null')
if code == 0:
	print("Rust compiler installed.")
	os.system('rustc --version')
else:
	has_error = True
	print("!!!ERROR!!! Rust compiler not found.")

if has_error:
	print('>>>>>>>>>> CHECK COMPLETED WITH ERRORS!')
else:
	print('>>>>>>>>>> CHECK COMPLETED')