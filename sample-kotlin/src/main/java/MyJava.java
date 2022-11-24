import java.lang.invoke.MethodHandle;

import com.polecatworks.hams.hams_h;

public class MyJava {
    public void howdy() {
        System.out.println("Howdy from JAVA");
        hams_h.hello_world();
        System.out.println("Howdy from JAVA - JUST DID hello_world via foreign and jextract");


    }
    public static void main(String[] args) {
        System.out.println("HELLO from JAVA");
    }
}
