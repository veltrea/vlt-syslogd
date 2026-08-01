# 同梱フォント

`shippori-subset.woff2` は **Shippori Mincho**（SemiBold）を、このサイトの
見出し・表頭に実際に出てくる文字だけへサブセットしたものです。

- 原本: https://github.com/fontdasu/ShipporiMincho
- ライセンス: SIL Open Font License 1.1（`OFL-ShipporiMincho.txt`）

再生成するときは、表示用の文字を HTML から集めてから:

    python3 -m fontTools.subset ShipporiMincho-SemiBold.ttf \
      --text-file=subset.txt \
      --output-file=shippori-subset.woff2 \
      --flavor=woff2 --layout-features='kern,palt,vert,vrt2,liga' \
      --no-hinting --desubroutinize
