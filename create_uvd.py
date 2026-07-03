#!/usr/bin/env python3
import json
import zipfile
import uuid

# Create disk structure
disk = {
    "id": str(uuid.uuid4()),
    "size": 10737418240,
    "verified": False,
    "os": [
        {
            "name": "linux",
            "version": "6.1.0",
            "architecture": "x86_64"
        },
        {
            "name": "linux",
            "version": "6.1.0",
            "architecture": "aarch64"
        }
    ]
}

# Write JSON to zip with compression
with zipfile.ZipFile("uvd.uvd", "w", zipfile.ZIP_DEFLATED) as zf:
    # Use compact JSON (no indentation) to minimize size
    json_content = json.dumps(disk, separators=(',', ':'))
    zf.writestr("disk.json", json_content)

print("✓ Created uvd.uvd archive (optimized)")
print("\nDisk content:")
print(json.dumps(disk, indent=2))
