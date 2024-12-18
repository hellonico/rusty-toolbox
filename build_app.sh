#export DEBUG=true

if [ -n "$DEBUG" ]; then
  echo "DEBUG mode"
fi

declare -a folders=(
#"app-ui-cute-llm"
#"tunelling"
#"app-convert-png-to-icns"
"app-chess"
#"app-ui-system-stats"
#"app-ui-probe-videos"
#"app-cli-convert-videos"
#"video-recorder-for-mum"
#"app-ui-open-link-in-1-hour"
#"ui-extract-audio"
#"ui-download-audio"
#"app-ui-gitlab-download-file"
)


if [ -n "$DEBUG" ]; then
  rm -fr target/debug/bundle/osx/*
else
  rm -fr target/release/bundle/osx/*
fi

for  folder  in  "${folders[@]}"
do
  cd "$folder"
  echo "export PACKAGE_ID=$folder ; cargo bundle --bin $folder --release"
  export PACKAGE_ID=$folder ;
  if [ -n "$DEBUG" ]; then
      cargo bundle --bin "$folder"
      # --target x86_64-apple-darwin
  else
      cargo bundle --bin "$folder" --release
      # --target x86_64-apple-darwin
  fi
  cd ..
done

if [ -n "$DEBUG" ]; then
  cp -fr target/debug/bundle/osx/* ~/Dropbox/CREATIVE/APPS
else
  cp -fr target/release/bundle/osx/* ~/Dropbox/CREATIVE/APPS
fi

# cargo fix --bin app-ui-open-link-in-1-hour