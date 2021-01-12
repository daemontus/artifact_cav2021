import os
import re
import time

re_bench = re.compile("\[v(\d+)\]__\[r(\d+)\]__\[(.+?)\]__\[(.+?)\].*\.aeon")
re_var_count = re.compile("\[v(\d+)\]")
#re_elapsed = re.compile("time for attractor detection=(\d+\.?\d*) seconds")
re_elapsed = re.compile("\s*(\d+\.?\d*) real.*")

def is_bench(benchmark):	
	return re_bench.match(benchmark) != None

def bench_cmp(benchmark):
	m = re_var_count.match(benchmark)
	return int(m.group(1))

benchmarks = filter(is_bench, os.listdir("./benchmarks_real_life"))
benchmarks = sorted(benchmarks, key=bench_cmp)


out_dir = "basic_" + str(int(time.time()))
os.mkdir(out_dir)

csv = open(out_dir + "/stats.csv", "w")
csv.write("Benchmark, Time[s]\n")

elapsed_times = {}
i = 1
for benchmark in benchmarks:
	bench_name = re_bench.match(benchmark).group(3)
	print("Starting "+bench_name+" "+str(i)+"/"+str(len(benchmarks)))
	in_file = "benchmarks_real_life/" + benchmark
	out_file = "./" + out_dir + "/" + str(i) + "_" + bench_name + ".txt"
	os.system("gtimeout 1h time ./target/release/algorithm_basic < " + in_file + " > " + out_file + " 2>&1")
	i = i + 1
	with open(out_file, 'r') as f:
		lines = f.read().splitlines()
		time_line = lines[-1]		
		if re_elapsed.match(time_line):
			print lines[-2]
			print lines[-3]
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