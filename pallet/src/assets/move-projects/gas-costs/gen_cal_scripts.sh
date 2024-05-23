#!/bin/sh
cd $(dirname $0)

MOVE_SRC="./sources/Calibration.move"
BASH_SH="./gen_smove_instr.sh"
ITERATIONS=25

function write_method() {
    printf "script {\n" >> $MOVE_SRC
    printf "    fun cal_gas_cost_$1(v0: u8" >> $MOVE_SRC
    if [ $1 -gt 0 ]
    then
        for i in $(seq 1 $1)
        do
            printf ", v$i: u8" >> $MOVE_SRC
        done
    fi
    printf ") {\n" >> $MOVE_SRC
    for i in $(seq 0 $1)
    do
        printf "        assert!(v$i == $i, $i);\n" >> $MOVE_SRC
    done
    printf "    }\n" >> $MOVE_SRC
    printf "}\n" >> $MOVE_SRC
}

function write_smove_cmd() {
    printf "\nsmove create-transaction -c build/gas-costs/bytecode_scripts/cal_gas_cost_$1.mv --args" >> $BASH_SH
    for i in $(seq 0 $1)
    do
        printf " u8:$i" >> $BASH_SH
    done
}

printf "" > $MOVE_SRC
printf "#!/bin/sh" > $BASH_SH
for i in $(seq 0 $(($ITERATIONS-1)))
do
    write_method $i
    write_smove_cmd $i
done
