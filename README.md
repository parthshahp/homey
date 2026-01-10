# Homey

A homepage for your server applications, written in Rust.

There is a docker-compose provided in the repo for easy install.

The config.json is where you add and remove apps.

### Configuration

```json
{
  "title": "string",
  "links": [
    {
      "name": "string",
      "url": "string",
      "altName": "string (optional)",
      "icon": "string (optional)",
    },
    {
      "name": "string",
      "url": "string",
      "altName": "string (optional)",
      "icon": "string (optional)",
    },
  ]
}
```

- The display name on the card will be the `altName` field if it exists, and `name` if it doesn't.
- The icons are sourced from [https://selfh.st/icons](https://selfh.st/icons/) using the `name` field. This behavior can be overridden by providing a link in the `icon` field.
