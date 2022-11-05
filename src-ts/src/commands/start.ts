import type { Arguments, CommandBuilder } from 'yargs';

// import {ffi} from 'ffi-napi';

import ffi from 'ffi-napi';
import ref from 'ref-napi';
var StructType = require('ref-struct-di')(ref);

type Options = {
    name: string;
    upper: boolean | undefined;
};

export const command: string = 'start <name>';
export const desc: string = 'Start <name> with Hello';

export const builder: CommandBuilder<Options, Options> = (yargs) =>
    yargs
        .options({
            upper: { type: 'boolean' },
        })
        .positional('name', { type: 'string', demandOption: true });

export const handler = (argv: Arguments<Options>): void => {
    const { name, upper } = argv;
    const greeting = `Hello, ${name}!`;
    process.stdout.write(upper ? greeting.toUpperCase() : greeting);


    var cmetadata = 'void';
    var cmetadataPtr = ref.refType(cmetadata);
    var externCRecord = 'void';
    var externCRecordPtr = ref.refType(externCRecord);

    const LogParam = StructType({
        enabled:  ffi.Function('bool', [cmetadataPtr]),
        log: ffi.Function('void', [externCRecordPtr]),
        flush: ffi.Function('void', []),
        level: ref.types.uint,
        // level: ref.refType(ref.types.uint),
    });


    console.log("HELLO");

    const ffilib = ffi.Library('libhams',
    {
        hello_world: ['void', []],
        hello_node: ['int', []],
        hams_logger_init: ['int', ['pointer']],
        hams_init: [ 'pointer' , ['string']],
        hams_start: ['int', ['pointer']],
        hams_free: ['int', ['pointer']],
    }
    );

    var ben = ffilib.hams_init("hello");

    ffilib.hams_start(ben);

    ffilib.hams_free(ben);

    console.log("freed");


    var funcPtr = ffi.Callback('int', [ 'int' ], (my_input: number) => {
        console.log("run my Math.abs");
        return Math.abs(my_input);
    });

    var func = ffi.ForeignFunction(funcPtr, 'int', [ 'int' ]);

    var my_return = func(-3);
    console.log("my funv reply is ", my_return);


    // var time_t = ref.types.long;
    // var suseconds_t = ref.types.long;

    // // // define the "timeval" struct type
    // var timeval = StructType({
    //     tv_sec: time_t,
    //     tv_usec: suseconds_t
    // });

    // var tv = new timeval({ tv_sec: 1, tv_usec: 2});

    // console.log("XXX", tv.tv_sec);

    console.log("ABOUT to say hello");
    ffilib.hello_world();
    ffilib.hello_world();

    console.log("I did HELLO");
    var hello_node_reply = ffilib.hello_node();

    console.log("hello_node = ", hello_node_reply);




    var c_log_enabled = ffi.Callback('bool', [cmetadataPtr],
        function(cmetadata) {
            return true;
        }
    );
    var c_log_log = ffi.Callback('void', [externCRecordPtr],
        function(cmetadata) {
            console.log("Logging using c_log_log via nodejs");
            const loc = "here";
            const message = "this message";
            console.log(`Node Log(${loc}): ${message}`);
        }
    );
    var c_log_flush = ffi.Callback('void', [],
        function() {
            console.log("flushing");
        }
    );

    var test_c_log_flush = ffi.ForeignFunction(c_log_flush, 'void', []);
    console.log("testing c_log_flush");
    test_c_log_flush();
    console.log("Tested c_log_flush");




    var myLog = new LogParam({
        enabled: c_log_enabled,
        log: c_log_log,
        flush: c_log_flush,
        level: 3
    });
    console.log("Ready to register LOGGING");
    var log_reply = ffilib.hams_logger_init(myLog.ref());

    console.log("Logging registered");

    class Hams {
        private name: string;
        private hams: any;

        constructor(public cname: string) {
            this.name = cname;
            this.hams = ffilib.hams_init(cname);
            console.log("constructed");
        }

        start() {
            console.log("starting");
            ffilib.hams_start(this.hams);
        }

        free() {
            console.log("free");
            ffilib.hams_free(this.hams);
        }
    }

    var me = new Hams("hello");

    me.start();

    me.free();





    process.exit(0);
};
