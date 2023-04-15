# samcut
`samcut` is `cut` for SAM format text.

## Install

```bash
conda install -c bioconda samcut
```

## Examples

Basics:

```bash
$ samtools view in.bam | head -n4 | samcut -H | column -t
qname                                        flag  rname  pos     mapq  cigar  rnext  pnext   tlen  seq                                   qual
HISEQ-2500-1:47:C5V27ANXX:1:1101:12050:5400  99    chr1   629922  23    36M    =      629946  60    TCATTAATAATCATAATGGCTATAGCAATAAAACTA  BBBBBFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
HISEQ-2500-1:47:C5V27ANXX:1:1101:16204:8196  163   chr1   629626  23    36M    =      629634  44    TCCTTCCCGTACTAATTAATCCCCTGGCCCAACCCG  //<<<FB/FBBFFB//</<B</FBBBFF<<B#####
HISEQ-2500-1:47:C5V27ANXX:1:1101:16204:8196  83    chr1   629634  15    36M    =      629626  -44   GTACTAATTAATCCCCTGGCCCAACCCGTCATCTAC  FFFFF<FFFF<7F/BFB<B<FF<FB<B<BFFBB<B/
HISEQ-2500-1:47:C5V27ANXX:1:1101:8202:71354  99    chr1   629937  23    36M    =      629964  63    ATGGCTATAGCAATAAAACTAGGAATAGCCCCCTTT  BBB<BFFFFFBFFFFFFFFFFFFFFFFFFFFFFFFF
```

Flags:

```bash
$ samtools view in.bam | head -n4 | samcut -H flag read1 read2 paired dup flags | column -t
flag  read1  read2  paired  dup  flags
99    1      0      1       0    paired,proper_pair,mreverse,read1
163   0      1      1       0    paired,proper_pair,mreverse,read2
83    1      0      1       0    paired,proper_pair,reverse,read1
99    1      0      1       0    paired,proper_pair,mreverse,read1
```

Expand all flags:

```bash
$ samtools view in.bam | head -n4 | samcut -H flag flagss flags | column -t
flag  paired  proper_pair  unmap  munmap  reverse  mreverse  read1  read2  secondary  qcfail  dup  supplementary  flags
99    1       1            0      0       0        1         1      0      0          0       0    0              paired,proper_pair,mreverse,read1
163   1       1            0      0       0        1         0      1      0          0       0    0              paired,proper_pair,mreverse,read2
83    1       1            0      0       1        0         1      0      0          0       0    0              paired,proper_pair,reverse,read1
99    1       1            0      0       0        1         1      0      0          0       0    0              paired,proper_pair,mreverse,read1
```

Get stats about flag:

```bash
$ function count { sort | uniq -c | sort -nrk1,1; }
$ cols="dup read1 read2 paired"
$ (echo count $cols; samtools view in.bam | samcut $cols | count | head -n5) | column -t
count  dup  read1  read2  paired
3408   1    1      0      1
3317   1    0      1      1
1656   0    1      0      1
1619   0    0      1      1
```

Get tags:

```bash
$ samtools view in.bam | head -n5 | samcut -H qname SM MD MQ MC XA | column -t
qname                                         SM  MD  MQ  MC   XA
HISEQ-2500-1:47:C5V27ANXX:1:1101:12050:5400   23  36  22  36M  chrM,+4752,36M,1;
HISEQ-2500-1:47:C5V27ANXX:1:1101:16204:8196   23  36  15  36M  chrM,+4456,36M,1;
HISEQ-2500-1:47:C5V27ANXX:1:1101:16204:8196   0   36  23  36M  chrM,-4464,36M,0;
HISEQ-2500-1:47:C5V27ANXX:1:1101:8202:71354   23  36  22  36M  chrM,+4767,36M,1;
HISEQ-2500-1:47:C5V27ANXX:1:1102:11490:86472  0   36  15  36M  .
```

Histogram of flags and tags:

```bash
$ samtools view in.bam | samcut NM read1 proper_pair | count | column -t
4347  0  0  1
4202  0  1  1
774   1  1  1
430   1  0  1
159   2  0  1
88    2  1  1
```


## Manual

```
$ samcut --help
```

```
samcut is cut for sam: `samtools view in.bam | samcut`. See --help for examples.

Print the standard 11 columns (qname, flag, ..., qual) with a header row:
    $ samtools view in.bam | samcut -H

Print qname, cigar, pos, and tlen only:
    $ samtools view in.bam | samcut qname cigar pos tlen

Also print a specific tag:
    $ samtools view in.bam | samcut qname cigar pos tlen MD

Separate flag into columns for each bit:
    $ samtools view in.bam | samcut -H qname flagss flags

Get a histogram of (read1, secondary, supplementary) flag values:
    $ samtools view in.bam | samcut read1 secondary supplementary | sort | uniq -c

Usage: samcut [OPTIONS] [FIELDS]...

Arguments:
  [FIELDS]...
          Select only these fields. Example: `samcut n qname tlen read1 MD`. See --help for details.
          If not supplied, `std` is used.
          
          Standard keys:
              key              description
              ----------------------------------------------------------------------------
              qname            query template name
              flag             bitwise flag
              rname            reference sequence name
              pos              1-based leftmost mapping position
              mapq             mapping quality
              cigar            cigar string
              rnext            ref. name of the mate/next read
              pnext            position of the mat/next read
              tlen             observed template length
              seq              segment sequence
              qual             ascii of phred-scaled base quality+33
          
          Flag keys:
              key              description
              ----------------------------------------------------------------------------
              paired           paired-end (or multiple-segment) sequencing technology
              proper_pair      each segment properly aligned according to the aligner
              unmap            segment unmapped
              munmap           next segment in the template unmapped
              reverse          SEQ is reverse complemented
              mreverse         SEQ of the next segment in the template is reversed
              read1            the first segment in the template
              read2            the last segment in the template
              secondary        secondary alignment
              qcfail           not passing quality controls
              dup              PCR or optical duplicate
              supplementary    supplementary alignment
          
          Speical keys:
              key              description
              ----------------------------------------------------------------------------
              n                One-based index for the input stream
              std              Expands to the standard 11 columns (qname, flag, ..., qual)
              flags            Humarn readable flag ("paired,proper_pair,mreverse,read1")
              flagss           Expands to the 12 flag names (from `samtool flags`)

Options:
  -H, --header
          Print a header row with column names

  -d, --delim <DELIM>
          Character to use as delimiter for output
          
          [default: "\t"]

  -i, --fill <FILL>
          String to fill missing values with (tags are optional and can be missing)
          
          [default: .]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
