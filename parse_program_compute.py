import re
import configparser
import os


def read_compute_units(filename):
    lines = []
    call = ""
    with open(filename, "r") as file:
        for line in file:
            call_match = re.search(r"Program log: Instruction: \w+", line)
            match = re.search(
                r"Program \w+ consumed (\d+) of 200000 compute units", line
            )
            if call_match:
                call = line.split("Instruction:")[1].strip()
            if match:
                lines.append(
                    {call: int(line.split("consumed")[1].split("of")[0].strip())}
                )
    return lines


program_log_directory = ".anchor/program-logs"
for program_log_file in os.listdir(program_log_directory):
    print(f"Program log file of data: {program_log_file}")
    result = read_compute_units(f"{program_log_directory}/{program_log_file}")
    for line in result:
        print(line)
