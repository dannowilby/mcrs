const path = require('path');
const express = require('express');
const app = express();
const port = 3000;

app.use('/pkg', express.static('pkg'));

app.get('/', function(req, res) {
  res.sendFile(path.join(__dirname, '/index.html'));
});

app.listen(port, () => {
  console.log(`Connect to http://localhost:${port}/ to play.`);
});

