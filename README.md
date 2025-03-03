# Sunzip - 7Z Compact/Extract Tool


## Features

- Cyclic-Redundancy-Check Verification when trying to existing extract files

## USAGE Example


> Given dummy files `data*.txt`

```
rm -f data-{2,3,4,5,6,7}.txt;
rm -f *.txt
cat >gendata.sh <<EOF
#!/bin/bash
for s in \$(seq 2 7); do
    target="data-\${s}.txt"
    echo -n > \$target
    echo "\$(seq 10 | xargs | sed 's/[[:space:]]//g' | sed 's/[[:digit:]]/-/g')" >> \$target
    echo "\$target" >> \$target
    echo -e "\$(seq 10 | xargs | sed 's/[[:space:]]//g' | sed 's/[[:digit:]]/-/g')\n\n" >> \$target
    seq \$((s * 7)) | sed "s,^,[\$(date +"%Y/%m/%d-%H:%M:%S")],g" >> \$target
    echo "Generated dummy data in: \$target"
done
EOF
bash gendata.sh
```


### Compact Files into Archive


```
s7zip -o archive.7z data*.txt
```

### Extract 7z Archive

```
sunzip archive.7z
```

> or

```
s7unzip archive.7z
```
