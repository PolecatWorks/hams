import type { Arguments, CommandBuilder } from 'yargs';

import ffi from 'ffi-napi';
import ref from 'ref-napi';
var StructType = require('ref-struct-di')(ref);

type Options = {
};

export const command: string = 'start';
export const desc: string = 'Start hams based service';

export const builder: CommandBuilder<Options, Options> = (yargs) =>
    yargs;

export const handler = (argv: Arguments<Options>): void => {

    const RustString = StructType({
        ptr: ref.types.CString,
        cap: ref.types.uint64,
        len: ref.types.uint64,
    });

    const RustStr = StructType({
        ptr: ref.types.CString,
        len: ref.types.uint64,
    });

    var ExternCMetadata = StructType({
        level: ref.types.int64,
        target: RustStr,
    });

    const ExternCRecord = StructType({
        metadata: ExternCMetadata,
        message: RustString,
        module_path: RustStr,
        file: RustStr,
        line: ref.types.int64
    });

    var ExternCRecordPtr = ref.refType(ExternCRecord);

    const LogParam = StructType({
        enabled:  ffi.Function('bool', [ExternCMetadata]),
        log: ffi.Function('void', [ExternCRecordPtr]),
        flush: ffi.Function('void', []),
        level: ref.types.uint,
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


    console.log("ABOUT to say hello");
    ffilib.hello_world();
    ffilib.hello_world();

    console.log("I did HELLO");
    var hello_node_reply = ffilib.hello_node();

    console.log("hello_node = ", hello_node_reply);




    var c_log_enabled = ffi.Callback('bool', [ExternCMetadata],
        function(cmetadata) {
            return true;
        }
    );
    var c_log_log = ffi.Callback('void', [ExternCRecordPtr],
        function(recordptr: any) {
            const record: typeof ExternCRecord = recordptr.deref();
            const loc = record.module_path.ptr;
            const message = record.message.ptr;
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
