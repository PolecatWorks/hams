#include <stdio.h>
#include <config.h>
#include <hams.h>

bool c_log_enabled(ExternCMetadata logdata) {
    return true;
}

void c_log_log(const struct ExternCRecord* logdata) {
    printf("C Log: %.*s\n", (int)logdata->message.len, logdata->message.ptr);
}

void c_log_flush() {
    printf("Flushing");
}


// extern void hello_world(); // declare the Rust function
int main(void)
{
    struct LogParam c_log = {
        c_log_enabled,
        c_log_log,
        c_log_flush,
        ExternCLevelFilter_Info
        };


    hams_logger_init(c_log);


    void* hams = hams_init("hello");

    hams_start(hams);

    hams_free(hams);

    printf("DONE\n");
}
