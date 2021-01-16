const app = require('express')();
const http = require('http').createServer(app);
const io = require('socket.io')(http);
const net = require('net');
const Convert = require('ansi-to-html');

const convert = new Convert();

function color(s) {
    return convert.toHtml(s);
}


app.get('/', (req, res) => {
    res.sendFile(__dirname + '/index.html');
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
        client.write(msg + '\n');
    });
});

io.on('disconnect', (socket) => {
    console.log('a user disconnected');
});

http.listen(3000, () => {
    console.log('listening on *:3000');
});
