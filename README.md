# fastLink
A tool to make symlink fastly and smartly, support regex  
一个智能且方便的符号链接创建工具，支持regex  

For Windows (at present)

如果你处于Windows的开发者模式，则不需要`sudo`或者`管理员权限`  
Do not need `sudo` or `administrator` if you are in the Developer Mode.  

![example](image/README/example.png)

## Usage (not up to date yet)
```cmd
Usage: fastlink.exe [OPTIONS] <SRC> [DST]

Arguments:
  <SRC>
          源文件/源目录路径

  [DST]
          目标路径，可选，区分文件拓展名。 为空则自动以<SRC>路径名称填充；当<SRC>为文件，[DST]为目录时，自动以<SRC>路径名称填充

Options:
  -k, --keep-extention
          自动保留<SRC>的文件拓展名到[DST]。(不会去除) 保留拓展名之后可以通过对符号链接双击、运行等操作让系统使用默认应用打开或执行。

      --make-dir
          自动创建不存在的目录

          [aliases: --md]

  -q, --quiet
          只输出warn与error level的日志

      --debug
          输出debug level的日志

      --regex <REGEX>
          对<SRC>内容应用正则表达式，匹配项将于[DST]相应创建， 若启用make_dir参数，则还会尝试对<SRC>的子目录以及更深层(默认最大4层)进行匹配并创建， 若要限制深度，使用--re-max-depth参数。 匹配的路径不受--keep_extention参数 影响。

          [aliases: --re]

      --re-max-depth <RE_MAX_DEPTH>
          限制regex匹配的最大深度，启用make_dir参数时，默认4层，否则为1层, 传入0表示没有层数限制. 该参数数值非负

          [aliases: --re-depth]

      --only-file
          只处理文件，同时传入only-dir则出错

          [aliases: --F]

      --only-dir
          只处理目录，同时传入only-file则出错

          [aliases: --D]

      --re-follow-links
          re匹配过程中，深入读取符号链接进行匹配

          [aliases: --follow-links, --follow-link]

      --re-no-check
          取消re匹配后，创建链接前的用户手动检查阶段

          [aliases: --no-check]

      --re-output-flatten
          对于re匹配的后所有内容，不按照原本目录（镜像）创建链接， 而是直接创建到[DST]中。 如果匹配的文件名有重复，则会拒绝创建并报错

          [aliases: --flatten]

      --overwrite-links
          覆盖同名已存在的符号链接，与--skip-exist-links互斥

          [aliases: --overwrite, --overwrite-link]

      --overwrite-broken-link
          --overwrite-links的较弱版本，但优先级高于--skip-exist-link，只覆盖损坏的符号链接. 默认为true, 暂不支持关闭

          [aliases: --overwrite-broken]

      --skip-exist-links
          针对[DST]，跳过同名已存在的符号链接，与--overwrite-links互斥

          [aliases: --skip-exist, --skip-exists, --skip-exist-link, --skip-exist-links]

      --skip-broken-src-links
          针对<SRC>，跳过损坏的符号链接. 默认为true, 暂不支持关闭

          [aliases: --skip-broken, --skip-broken-link, --skip-broken-links]

      --save-log <SAVE_LOG>
          在目标路径输出/保存/导出本次处理日志 若路径不存在，则将当前工作目录并重命名为fastlink-%y-%m-%d-%h-%m-%s.log

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Example
```bash
    # 在当前目录创建一个名为document的符号链接
    fastlink document.txt

    # 在当前目录创建一个名为img-link.jpg的符号链接
    fastlink image.jpg img-link -k

    # 在当前目录的子目录tmp中创建名为output.csv的符号链接，若tmp目录不存在将退出
    fastlink data.csv tmp/output --keep-extention

    # 同上，但添加--make-dir或--md参数选项将自动创建目录
    fastlink data.csv tmp/output --keep-extention --md

    # 在当前目录的父目录创建名为data符号链接，指向data.csv (不建议, Not Recommended)
    fastlink data.csv ../
    
    # 为./test-dir目录中所有满足.*\.txt正则表达式的路径（即所有txt文件） 创建链接到output目录中
    fastlink ./test-dir output --re .*\.txt 

    # ./test-dir目录及其子目录或更深目录中所有txt文件 -> 镜像目录创建链接到output目录中
    fastlink ./test-dir output --re .*\.txt --only-file

    # 将./test-dir目录及其子目录或更深目录中所有txt文件 -> 直接创建链接到output目录中，不包含文件夹（可包含对文件夹的符号链接）
    fastlink ./test-dir output --re .*\.txt --flatten
```

## 未来计划
- [ ] 支持 Unix | support Unix  
- [ ] 完善测试  
- [ ] release轻量版二进制程序（不支持re等特性）  

## Feedback
If something not excepet happened, open an issue and paste the log wiht `--debug` argument  

如果发生了一些预期之外的问题，提个issue，记得贴上带`--debug`参数时的日志
