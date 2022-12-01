from os import listdir
from os.path import isfile, join
import os 
RESULTS_DIR = "/Users/elsayedzaki/Desktop/rust projects/dist/Results"
dir_list = os.listdir(RESULTS_DIR)
avg_response_time = 0
avg_failures= 0
for file in dir_list:
    if file.endswith("secondcopy.txt"):
        data = open(RESULTS_DIR + "/" + file, "r")
        mystring = data.readline()
        line = mystring.split(" ")
        avg_response_time += float(line[3])
        num = line[12].replace("\n", "")
        avg_failures += float(line[12])

print("Average response time: " + str(avg_response_time/500)+ " ms")
print("Average failures per client: " + str(avg_failures/500))