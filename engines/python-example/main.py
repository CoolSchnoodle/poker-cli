import sys

if len(sys.argv) < 3:
    raise Exception("Unexpected invocation")

if sys.argv[1] == "START":
    print("Python example bot")
    exit(0)

if len(sys.argv) < 8:
    raise Exception("Unexpected invocation")

executable_name = sys.argv[0]
min_bet_to_not_fold = int(sys.argv[1])
turn_time_ms = int(sys.argv[2])
time_bank_ms = int(sys.argv[3])
game_params_file = open(sys.argv[4])
game_state_file = open(sys.argv[5])
game_actions_file = open(sys.argv[6])
temp_dir_path = sys.argv[7]

print("FOLD", flush=True)

# cleanup, logging
print("long-term logging", file=sys.stderr)
