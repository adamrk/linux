obj-$(CONFIG_RUST_HELLO_WORLD) += test.o

#test.o: test.rs
	#rustc -C panic=abort -O --emit=obj --target=x86_64-unknown-linux-gnu test.rs

#default: test.o

clean:
	rm -f *.o *.a *.order *.mod *.ko *.mod.c
