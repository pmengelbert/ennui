var socket = io();

var form = document.getElementById('form');
var input = document.getElementById('command');

var id = -1;

function fd(event) {
  event.preventDefault();
  if (input.value) {
    socket.emit('chat message', input.value);
    input.value = '';
  }
}

form.addEventListener('submit', fd);

function processOutput(current, rawOutput) {
  let currentArray = current.split("\n");
  let outputArray = rawOutput.split("\n");
  let totalOutput = currentArray.concat(outputArray);
  const maxLines = 25;

  if (totalOutput.length > maxLines) {
    totalOutput = totalOutput.slice(totalOutput.length - maxLines, totalOutput.length);
  }

  let coloredOutput = totalOutput.join("\n");
  return coloredOutput
}

socket.on('chat message', (msg) => {
  console.log('HERE');
  let current = document.getElementById("result").innerHTML;
  let rawOutput = msg;
  let coloredOutput = processOutput(current, rawOutput);

  document.getElementById("result").innerHTML = coloredOutput;
});

socket.on('id', (pid) => {
  console.log("GOT IT");
  id = pid;
  console.log(id);
}); 

