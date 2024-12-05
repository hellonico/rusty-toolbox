
#function build() {
#    cargo bundle --bin $1
#
#}
##
declare -a folders=(
#"app-ui-cute-llm"
#"tunelling"
#"app-convert-png-to-icns"
"app-chess"
#"app-ui-system-stats"
#"app-ui-probe-videos"
#"app-cli-convert-videos"
# "video-recorder-for-mum"
#"app-ui-open-link-in-1-hour"
)


for  folder  in  "${folders[@]}"
do
  cd "$folder"
  echo "export PACKAGE_ID=$folder ; cargo bundle --bin $folder"
  export PACKAGE_ID=$folder ; cargo bundle --bin $folder
  cd ..
done

cp -fr target/debug/bundle/osx/* ~/Dropbox/CREATIVE/APPS
