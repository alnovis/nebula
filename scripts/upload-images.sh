#!/bin/bash
# Upload images to Cloudinary

CLOUD_NAME="${CLOUDINARY_CLOUD_NAME:-ddkzhz9b4}"
API_KEY="${CLOUDINARY_API_KEY}"
API_SECRET="${CLOUDINARY_API_SECRET}"

if [ -z "$API_KEY" ] || [ -z "$API_SECRET" ]; then
    echo "Error: CLOUDINARY_API_KEY and CLOUDINARY_API_SECRET must be set"
    exit 1
fi

IMAGES_DIR="static/images"

for img in "$IMAGES_DIR"/*.webp; do
    [ -f "$img" ] || continue

    filename=$(basename "$img" .webp)
    public_id="nebula/$filename"

    echo "Uploading $img as $public_id..."

    timestamp=$(date +%s)
    signature=$(echo -n "public_id=$public_id&timestamp=$timestamp$API_SECRET" | sha1sum | cut -d' ' -f1)

    curl -s -X POST "https://api.cloudinary.com/v1_1/$CLOUD_NAME/image/upload" \
        -F "file=@$img" \
        -F "public_id=$public_id" \
        -F "timestamp=$timestamp" \
        -F "api_key=$API_KEY" \
        -F "signature=$signature" \
        | jq -r '.secure_url // .error.message'
done

echo "Done!"
