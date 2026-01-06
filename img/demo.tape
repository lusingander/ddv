Set Shell zsh

Output ./img/demo.gif

Set FontSize 15
Set Width 1000
Set Height 600
Set Padding 15
Set WindowBar Colorful
Set Theme "nord"

Hide

Type@10ms "export AWS_ACCESS_KEY_ID=dummy"
Enter
Type@10ms "export AWS_SECRET_ACCESS_KEY=dummy"
Enter
Type@10ms "./target/debug/ddv --endpoint-url http://localhost:8000"
Enter
Sleep 2s

Show

Sleep 1s
Type "j"
Sleep 500ms
Screenshot "./img/table-list-list.png"
Tab
Sleep 500ms
Screenshot "./img/table-list-detail-kv.png"
Type "v"
Sleep 1s
Screenshot "./img/table-list-detail-json.png"
Type@100ms "jjjjjjjjjjjjjjjjjjjjjjjjj"
Sleep 1s
Type "v"
Sleep 500ms
Tab
Sleep 500ms
Type "/"
Sleep 500ms
Type@200ms "sensor"
Sleep 200ms
Screenshot "./img/table-list-filtering.png"
Enter
Sleep 500ms
Type "j"
Sleep 1s
Enter

Sleep 3s
Screenshot "./img/table.png"
Type@200ms "jjjjj"
Sleep 1s
Type@200ms "lll"
Sleep 500ms
Type@200ms "++++"
Sleep 500ms
Type@200ms "llllllll"
Sleep 1s
Type@500ms "fff"
Sleep 500ms
Type@200ms "hhh"
Type "e"
Sleep 1s
Type "n"
Sleep 500ms
Screenshot "./img/table-expand-attr.png"
Type@200ms "jjjjj"
Sleep 500ms
Type "e"
Sleep 500ms
Type "^"
Sleep 500ms
Type@500ms "jjj"
Sleep 500ms
Type "h"
Sleep 1s

Type "/"
Sleep 500ms
Type@200ms "critical"
Screenshot "./img/table-filtering.png"
Sleep 500ms
Enter
Sleep 1s

Enter

Sleep 2s
Screenshot "./img/item-kv.png"
Type "w"
Sleep 500ms
Type "n"
Sleep 1s
Type "v"
Sleep 1s
Screenshot "./img/item-plain-json.png"
Type "v"
Sleep 500ms
Screenshot "./img/item-raw-json.png"
Type@100ms "jjjjjjjj"
Sleep 1s
Type "c"
Sleep 2s
Backspace

Sleep 2s
Type "i"
Sleep 3s
Screenshot "./img/table-insight.png"
Backspace
Sleep 2s
