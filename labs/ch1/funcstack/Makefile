TARGET := funcstack

LDFLAGS := -Wl,-L/usr/lib/x86_64-linux-gnu -lunwind -lunwind-x86_64

all: build

build:
	gcc -g $(TARGET).c -o $(TARGET) $(LDFLAGS)

clean:
	rm -rf $(TARGET)


run: build
	./$(TARGET)

.PHONY: build clean run
