

package com.polecatworks.kotlin.samples
import com.sun.jna.*

// https://www.baeldung.com/java-jna-dynamic-libraries
// https://java-native-access.github.io/jna/3.5.0/javadoc/index.html


class RustStr (
    @JvmField var ptr: Pointer,
    @JvmField var len: NativeLong,
): Structure(Structure.ALIGN_GNUC) {
    override fun getFieldOrder() = listOf("ptr", "len",)
}

class RustString (
    @JvmField var ptr: Pointer,
    @JvmField var cap: NativeLong,
    @JvmField var len: NativeLong,
): Structure(Structure.ALIGN_GNUC) {
    override fun getFieldOrder() = listOf("ptr", "cap", "len",)
}


class ExternCMetadata (
    @JvmField var level: NativeLong,
    @JvmField var target: RustStr,
): Structure(Structure.ALIGN_GNUC) {
    override fun getFieldOrder() = listOf("level", "target")
}



class ExternCRecord (
    @JvmField var metadata: ExternCMetadata,
    @JvmField var message: RustString,
    @JvmField var module_path: RustStr,
    @JvmField var file: RustStr,
    @JvmField var line: NativeLong,
): Structure(Structure.ALIGN_GNUC) {
    override fun getFieldOrder() = listOf("metadata", "message", "module_path", "file","line")
}



interface CHams : Library {
    // CHams INSTANCE = Native.load("hams", CHams))

    fun hello_world()
    fun hello_node(): Int
    // fun hams_logger_init(param: LogParam)
}



class HamsJni constructor() {
    init {
        println("hello Hams")
        System.loadLibrary("hams")
    }

    fun testMe(num: Int): Int {
        println("reading value $num")
        return 3
    }

    external fun foo(x: Int): Double

    companion object {
        init {
            println("loading object")
        }
    }
}
