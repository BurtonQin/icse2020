set term pdf
set output 'rq01-bb_freq.pdf'
set title "Unsafe Basic Blocks Frequency"
set xlabel "unsafe basic blocks/total"
set ylabel "%crates"
set xrange [0:1]
set yrange [0:30]
plot  "data/rq01-1.txt" with points pt 2 title 'Basic Blocks Frequency'

set term pdf
set output 'rq01-freq.pdf'
set title "Unsafe Basic Blocks Frequency"
set xlabel "unsafe basic blocks/total"
set ylabel "%crates"
set xrange [0:1]
set yrange [0:30]
plot  "data/rq01-3.txt" with points pt 2 title 'Blocks Frequency'


  
