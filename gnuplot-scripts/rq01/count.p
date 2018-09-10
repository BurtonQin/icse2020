set term pdf
set output 'rq01-bb_count.pdf'
set title "Unsafe Basic Blocks Count"
set xlabel "unsafe basic blocks/total"
set ylabel "% crates"
set logscale x
plot  "data/rq01-2.txt" with points title 'Unsafe Basic Blocks Count'

set term pdf
set output 'rq01-count.pdf'
set title "Unsafe Blocks Count"
set xlabel "unsafe blocks/total"
set ylabel "% crates"
set logscale x
plot  "data/rq01-4.txt" with points title 'Unsafe Blocks Count'



   
