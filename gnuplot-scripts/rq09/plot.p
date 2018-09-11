set term pdf
set output 'rq09-calls.pdf'
set title "Unsafe Function Calls from Normal Function"
set xlabel "unsafe function calls from normal functions"
set ylabel "% crates"
set logscale x
plot  "data/unsafe_calls.data" with points pt 2 title 'Unsafe Functions Calls Count'

set term pdf
set output 'rq09-raw.pdf'
set title "Raw Pointer Deref from Normal Function"
set xlabel "raw pointer deref from normal functions"
set ylabel "% crates"
set logscale x
plot  "data/deref_raw.data" with points pt 2 title 'Raw Pointer Deref Count'

set term pdf
set output 'rq09-static.pdf'
set title "Use of Static from Normal Function"
set xlabel "use of static from normal functions"
set ylabel "% crates"
set logscale x
plot  "data/static.data" with points pt 2 title 'Static Access Count'



   
