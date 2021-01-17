const app = require('express')();
const http = require('http').createServer(app);
const net = require('net');
const Convert = require('ansi-to-html');
const fs = require('fs');
const https = require('https');

function run(srv, port) {
	const io = require('socket.io')(srv);

	const convert = new Convert();

	function color(s) {
	  return convert.toHtml(s);
	}

	app.get('/', (req, res) => {
	  res.sendFile(__dirname + '/index.html');
	});

	app.get('/command.js', (req, res) => {
	  res.sendFile(__dirname + '/command.js');
	});

	io.on('connection', (socket) => {
	  let id = Math.floor(Math.random() * 101);
	  socket.emit('id', id);

	  console.log('a user connected');

	  var client = new net.Socket();
      client.connect(8089, '127.0.0.1', function() {
      console.log('connected to 8089');
	  });

	  client.on('data', function(data) {
      console.log(data.toString());
      socket.emit('chat message', color(data.toString()));
	  });

	  socket.on('chat message', (msg) => {
      if (msg.toString() === "q") {
        socket.emit('chat message', "DISCONNECTED");
        socket.disconnect();
      }
      client.write(msg + '\n');
	  });
	
	  socket.on('disconnect', (socket) => {
      client.write('q\n');
      console.log('a user disconnected');
	  });
	});

	srv.listen(port, () => {
	    console.log('listening on *:' + port);
	});
}

if (process.env.ENVIRONMENT === "production") {
	const privateKey = fs.readFileSync('/etc/letsencrypt/live/poobuttz.lol/privkey.pem', 'utf8');
	const certificate = fs.readFileSync('/etc/letsencrypt/live/poobuttz.lol/fullchain.pem', 'utf8');
	const credentials = {key: privateKey, cert: certificate};
	const httpsServer = https.createServer(credentials, app);
	run(httpsServer, 443);
} else {
	run(http, 3000);
}
