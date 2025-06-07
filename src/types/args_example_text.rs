// todo: 使用更舒服的方式创建EXAMPLE
#[cfg(feature = "slim")]
pub const EXAMPLE: &str = r#"
Example：
    // 在当前目录创建一个名为document的符号链接
    fastlink document.txt

    // 在当前目录创建一个名为img-link.jpg的符号链接
    fastlink image.jpg img-link -k

    // 在当前目录的子目录tmp中创建名为output.csv的符号链接，若tmp目录不存在将退出
    fastlink data.csv tmp/output --keep-extention

    // 同上，但添加--make-dir或--md参数选项将自动创建目录
    fastlink data.csv tmp/output --keep-extention --md

    // 在当前目录的父目录创建名为data符号链接，指向data.csv (不建议, Not Recommended)
    fastlink data.csv ../
"#;

#[cfg(not(feature = "slim"))]
pub const EXAMPLE: &str = r#"
Example：
    // 在当前目录创建一个名为document的符号链接
    fastlink document.txt

    // 在当前目录创建一个名为img-link.jpg的符号链接
    fastlink image.jpg img-link -k

    // 在当前目录的子目录tmp中创建名为output.csv的符号链接，若tmp目录不存在将退出
    fastlink data.csv tmp/output --keep-extention

    // 同上，但添加--make-dir或--md参数选项将自动创建目录
    fastlink data.csv tmp/output --keep-extention --md

    // 在当前目录的父目录创建名为data符号链接，指向data.csv (不建议, Not Recommended)
    fastlink data.csv ../

    // 为./test-dir目录中所有满足.*\.txt正则表达式的路径（即所有txt文件） 创建链接到output目录中
    fastlink ./test-dir output --re .*\.txt 

    // ./test-dir目录及其子目录或更深目录中所有txt文件 -> 镜像目录创建链接到output目录中
    fastlink ./test-dir output --re .*\.txt --only-file

    // 将./test-dir目录及其子目录或更深目录中所有txt文件 -> 直接创建链接到output目录中，不包含文件夹（可包含对文件夹的符号链接）
    fastlink ./test-dir output --re .*\.txt --flatten

    // 保存日志到指定文件
    fastlink document.txt --save-log my_log.log

    // 保存日志到默认路径（fastlink-YY-MM-DD-HH-MM-SS.log）
    fastlink data.csv tmp/output --md --save-log=""
"#;
