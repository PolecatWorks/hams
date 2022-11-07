#include <stdio.h>
#include <config.h>
#include <hams.h>

bool c_log_enabled(ExternCMetadata logdata) {
    return true;
}

void c_log_log(const struct ExternCRecord* logdata) {
    printf("C Log(%.*s): %.*s\n",
        (int)logdata->module_path.len,logdata->module_path.ptr,
        (int)logdata->message.len, logdata->message.ptr);
}

void c_log_flush() {
    printf("Flushing");
}


// extern void hello_world(); // declare the Rust function
int main(void)
{
    printf("sizeof(ExternCRecord) = %lu\n", sizeof(ExternCRecord));
    printf("sizeof(ExternCRecord*) = %lu\n", sizeof(ExternCRecord*));
    printf("sizeof(RustStr) = %lu\n", sizeof(RustStr));
    printf("sizeof(RustString) = %lu\n", sizeof(RustString));
    printf("sizeof(ExternCMetadata) = %lu\n", sizeof(ExternCMetadata));

    struct LogParam c_log = {
        c_log_enabled,
        c_log_log,
        c_log_flush,
        ExternCLevelFilter_Info
        };


    hams_logger_init(c_log);

    hello_world();

    Hams *hams = hams_init("hello");
    if (!hams) {
        printf("FAILED to init");
        return 1;
    }

    int start_reply = hams_start(hams);
    if (!start_reply) {
         printf("FAILED to start");
        return 2;
    }

    int free_reply = hams_free(hams);
    if (!free_reply) {
         printf("FAILED to free");
        return 3;
    }

    printf("DONE\n");
    return 0;
}
