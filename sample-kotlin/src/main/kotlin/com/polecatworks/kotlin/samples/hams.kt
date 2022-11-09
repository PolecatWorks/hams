

package com.polecatworks.kotlin.samples
import com.sun.jna.Library
import com.sun.jna.Native

// https://www.baeldung.com/java-jna-dynamic-libraries

interface CHams : Library {
    // CHams INSTANCE = Native.load("hams", CHams))

    fun hello_world();
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
    //public native fun stringFromJNI(): String

    //native fun unimplementedStringFromJNI(): String

    companion object {
        init {
            println("loading object")
        }
    }
}
