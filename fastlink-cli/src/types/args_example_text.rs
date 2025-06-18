// todo: 使用更舒服的方式创建EXAMPLE
#[cfg(feature = "fastlink-slim")]
pub const EXAMPLE: &str = r#"
Example：
    // 1. 在当前目录创建一个名为document.txt的符号链接（没有给出dst，直接用src文件名）
    fastlink some_where/document.txt

    // 2. 在当前目录创建一个名为img-link.jpg的符号链接
    // src为文件，使用dst作为链接名称，`-k`则自动从src获取拓展名追加到src
    fastlink image.jpg img-link -k

    // 3. dst有目录倾向, 在tmp目录下创建名为data.csv的符号链接
    // (目录倾向指的是路径以'\'或'/'结尾)
    fastlink data.csv tmp/

    // 4. 同上，--md将自动创建目录
    fastlink data.csv tmp/ --md

    // 5. 在当前目录的父目录创建名为data.csv的符号链接
    fastlink data.csv ../

    // 6. src为目录，dst有目录倾向，在backup中创建名为mydir的链接
    fastlink mydir/ backup/ --md
"#;

#[cfg(not(feature = "fastlink-slim"))]
pub const EXAMPLE: &str = r#"
Example：
    // 1. 在当前目录创建一个名为document.txt的符号链接（没有给出dst，直接用src文件名）
    fastlink some_where/document.txt

    // 2. 在当前目录创建一个名为img-link.jpg的符号链接
    // src为文件，使用dst作为链接名称，`-k`则自动从src获取拓展名追加到src
    fastlink image.jpg img-link -k

    // 3. dst有目录倾向, 在tmp目录下创建名为data.csv的符号链接
    // (目录倾向指的是路径以'\'或'/'结尾)
    fastlink data.csv tmp/

    // 4. 同上，--md将自动创建目录
    fastlink data.csv tmp/ --md

    // 5. 在当前目录的父目录创建名为data.csv的符号链接
    fastlink data.csv ../

    // 6. src为目录，dst有目录倾向，在backup中创建名为mydir的链接
    fastlink mydir/ backup/ --md

    // 7. 为./test-dir目录中所有满足.*\.txt正则表达式的路径（即所有txt文件） 创建链接到output目录中
    fastlink ./test-dir output --re .*\.txt 

    // 8. ./test-dir目录及其子目录或更深目录中所有文件 -> 镜像目录创建链接到output目录中
    fastlink ./test-dir output --re .* --only-file

    // 9. 将./test-dir目录及其子目录或更深目录中所有txt文件 -> 直接创建链接到output目录中，不包含原始目录结构（可包含对文件夹的符号链接）
    fastlink ./test-dir output --re .*\.txt --flatten

    // 10. 保存日志到指定文件
    fastlink document.txt --save-log my_log.log

    // 11. 保存日志到默认路径（fastlink-YY-MM-DD-HH-MM-SS.log）
    fastlink data.csv tmp/ --md --save-log=""
"#;
