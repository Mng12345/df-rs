This project makes a command line tool that can list the 10 biggest directories of the path you support(like `df` on linux). I am a beginner of rust, the tool may has performant problem, it takes almost 5 minutes to scan my `e:/` path on my Windows10 computer with 79GB size of `e` hard disk. Probably because the `e` disk contains a lot of frontend projects which contain a huge number of files in node_modules directory that slow the scanning speed.

Caution: The tool was only used and tested on Windows. If you have any problems on other system or have some good advices about speeding scanning performance, please let me know.

### install 

`
cargo install df-rs
`

### useage

`
df-rs e:/
`

### examples
```
PS C:\Users\Lenovo> df-rs.exe E:\IdeaProjects\
size                    dir
947mb                   ******
505mb                   rescript-example
500mb                   node-quant
186mb                   rxstate
167mb                   mng-easy-util
125mb                   stockexchangebacktest
123mb                   mng-color-picker
122mb                   mng-rx-state
117mb                   lyttest
109mb                   deliver-fileupload

PS C:\Users\Lenovo> df-rs.exe e:/
get size of e:/System Volume Information failed: Os { code: 5, kind: PermissionDenied, message: "拒绝访问。" }
size                    dir
18219mb                 vscode_proj
13842mb                 mywechat
6382mb                  ***
4924mb                  IdeaProjects
3440mb                  ***
2624mb                  software
2238mb                  ***
1778mb                  ***
1446mb                  ***
1331mb                  ***
```