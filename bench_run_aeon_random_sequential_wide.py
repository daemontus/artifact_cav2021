import os
import re
import time

re_bench = re.compile("(\d+)_(\d+)_(\d+)\.aeon")
re_var_count = re.compile("\d+_(\d+)")
#re_elapsed = re.compile("time for attractor detection=(\d+\.?\d*) seconds")
re_elapsed = re.compile("\s*(\d+\.?\d*) real.*")

def is_bench(benchmark):	
	return re_bench.match(benchmark) != None

def bench_cmp(benchmark):
	m = re_var_count.match(benchmark)
	return int(m.group(1))

benchmarks = filter(is_bench, os.listdir("./benchmarks_random_wide"))
benchmarks = sorted(benchmarks, key=bench_cmp)


out_dir = "sequential_random_wide_" + str(int(time.time()))
os.mkdir(out_dir)

csv = open(out_dir + "/stats.csv", "w")
csv.write("Benchmark, Time[s]\n")

elapsed_times = {}
i = 1
for benchmark in benchmarks:
	bench_name = str(benchmark) #re_bench.match(benchmark).group(3)
	print("Starting "+bench_name+" "+str(i)+"/"+str(len(benchmarks)))
	in_file = "benchmarks_random_wide/" + benchmark
	out_file = "./" + out_dir + "/" + str(i) + "_" + bench_name + ".txt"
	os.system("gtimeout 1h time ./target/release/algorithm_sequential < " + in_file + " > " + out_file + " 2>&1")
	i = i + 1
	with open(out_file, 'r') as f:
		lines = f.read().splitlines()
		time_line = lines[-1]
		print lines[-2]		
		if re_elapsed.match(time_line):
			print("Success: " + time_line)
			time = re_elapsed.match(time_line).group(1)
			elapsed_times[bench_name] = time
			csv.write(bench_name + ", " + str(time) + "\n")
		else:
			print("Failed!")
			elapsed_times[bench_name] = "Fail"
			csv.write(bench_name + ", " + "Fail" + "\n")
		csv.flush()

print "FINISHED"
csv.close()