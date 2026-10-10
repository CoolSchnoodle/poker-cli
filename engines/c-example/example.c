#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char** argv) {
    // shortest invocation is start invocation, with executable name, "START", temp dir path
    if (argc < 3) {
        // unknown invocation
        return 1;
    }
    if (strcmp(argv[1], "START") == 0) {
        // output bot name
        printf("C example bot\n");
    }
    if (argc < 8) {
        // unknown invocation
        return 1;
    }
    
    char* executable_name = argv[0];
    int min_bet_to_not_fold = atoi(argv[1]);
    int turn_time_ms = atoi(argv[2]);
    int time_bank_ms = atoi(argv[3]);
    char* game_params_path = argv[4];
    char* game_state_path = argv[5];
    char* game_actions_path = argv[6];
    char* temp_dir_path = argv[7];

    printf("FOLD\n");
    fflush(stdout);
    
    // cleanup, logging
    fprintf(stderr, "long-term logging\n");

    return 0;
}
