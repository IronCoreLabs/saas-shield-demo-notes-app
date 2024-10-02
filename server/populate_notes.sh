
function write_attachment {
  resp=$(curl -X POST -H "Content-Type: application/json" --cookie organization=$2 -d '{"filename": "'$1'"}' http://localhost:7654/api/attachments)
  attach_id=$(echo $resp | jq -r ".id")
  url=$(echo $resp | jq -r ".presigned_put_url")
  curl --upload-file $1 ${url}
  echo "Attachment $1 is ID ${attach_id}"
}

write_attachment "ramonesb.jpg" "notes-demo-1"
write_attachment "slf.jpg" "notes-demo-1"
write_attachment "squash.jpg" "notes-demo-2"
write_attachment "gourds.jpg" "notes-demo-2"



while read note; do
  curl -X POST -H "Content-Type: application/json" --cookie organization=notes-demo-1 -d "$note" http://localhost:7654/api/notes
done < notes_data_demo_1
while read note; do
  curl -X POST -H "Content-Type: application/json" --cookie organization=notes-demo-2 -d "$note" http://localhost:7654/api/notes
done < notes_data_demo_2
