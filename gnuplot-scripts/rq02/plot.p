set term pdf
set output 'rq02-count.pdf'
set title "Unsafe Functions"
set xlabel "unsafe functions"
set ylabel "% crates"
set logscale x
plot  "data/rq02-count.txt" with points title 'Unsafe Functions Count'

set term pdf
set output 'rq02-freq.pdf'
set title "Unsafe Functions Frequency"
set xlabel "% unsafe functions"
set ylabel "% crates"
set logscale x
plot  "data/rq02-freq.txt" with points title 'Unsafe Functions Frequency'



   
