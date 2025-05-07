# fastLink
A tool to make symlink fastly and smartly  
一个智能且方便的符号链接创建工具  

For Windows

## Usage
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

  -q, --quiet
          只输出warn与error level的日志

      --debug
          输出debug level的日志

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Example：
```bash
    # 在本目录创建一个名为document的符号链接
    fastlink document.txt

    # 在本目录创建一个名为img-link.jpg的符号链接
    fastlink image.jpg img-link -k

    # 在本目录的子目录tmp中创建名为output.csv的符号链接，若tmp目录不存在将自动创建并警告
    fastlink data.csv tmp/output --keep-extention

    # 在本目录的父目录创建名为data符号链接，指向data.csv (不建议, Not Recommended)
    fastlink data.csv ../
```

## Feedback
If something not excepet happened, open an issue and paste the log wiht `--debug` argument  

如果发生了一些预期之外的问题，提个issue，记得贴上带`--debug`参数时的日志