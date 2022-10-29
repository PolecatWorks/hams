#include <stdio.h>
#include <config.h>
#include <hams.h>

// extern void hello_world(); // declare the Rust function
int main(void)
{
    hello_world();

    void* hams = hams_init("hello");

    hams_start(hams);

    hams_free(hams);
    hello_world();
}
