run:
    echo Hello {{{{ koopa.name }}! > a.txt
    -cargo r -- a.txt b.txt -s project=koopa
    -cat b.txt
    rm a.txt
    -rm b.txt

x1:
    -cargo r -- .koopa/basic.py app.py -s project=override
    -cat app.py
    -rm app.py

x2:
    -cargo r -- du.vhd fifo.vhd --verbose
    -cat fifo.vhd
    -rm fifo.vhd

x3:
    -cargo r -- prj-cpp ./demo -s project=demo -s user="Darth Vader"
    -cd demo; just build
    -cd demo; just run
    -cd demo; just clean
    -rm -Rf ./demo