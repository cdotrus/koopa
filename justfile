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