.PHONY: all clean

all: ../README.md

../target/debug/samcut:
	cargo build

examples.md: ../target/debug/samcut
	bash examples.sh > examples.md

../README.md: template.md examples.md help.md
	bash render_template.sh template.md > ../README.md

help.md: ../target/debug/samcut
	samcut --help > help.md

clean:
	rm -f ../target/debug/samcut examples.md help.md ../README.md

.INTERMEDIATE: examples.md help.md
