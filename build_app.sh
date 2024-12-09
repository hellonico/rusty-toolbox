export DEBUG=true

if [ -n "$DEBUG" ]; then
  echo "DEBUG mode"
fi

declare -a folders=(
#"app-ui-cute-llm"
#"tunelling"
#"app-convert-png-to-icns"
#"app-chess"
#"app-ui-system-stats"
#"app-ui-probe-videos"
#"app-cli-convert-videos"
#"video-recorder-for-mum"
"app-ui-open-link-in-1-hour"
)


for  folder  in  "${folders[@]}"
do
  cd "$folder"
  echo "export PACKAGE_ID=$folder ; cargo bundle --bin $folder --release"
  export PACKAGE_ID=$folder ;
  if [ -n "$DEBUG" ]; then
      cargo bundle --bin "$folder"
  else
      cargo bundle --bin "$folder" --release
  fi
  cd ..
done

if [ -n "$DEBUG" ]; then
  cp -fr target/debug/bundle/osx/* ~/Dropbox/CREATIVE/APPS
else
  cp -fr target/release/bundle/osx/* ~/Dropbox/CREATIVE/APPS
fi

# cargo fix --bin app-ui-open-link-in-1-hour