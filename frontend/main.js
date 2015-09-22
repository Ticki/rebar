var server = "http://localhost:7272/query";
var resp_crates = [];

$(document).ready(function() {
    for(var i = 0; i < 500; i++) {
        $(".content").append("<tr><td id='" + i + "-vote'></td><td> <a  id='" + i + "-url' /></td><td id='" + i + "-desc'></td></tr>");
    }
    get_crates(function(n, id, host, username, name, desc) {
        $("#" + n + "-vote").html("<img height='32' width='32' onclick='upvote(" + id + ")' src='vote.png'/>");
        $("#" + n + "-url").text(username + "/" + name).attr("href", "https://github.com/" + username + "/" + name);
        $("#" + n + "-desc").text(desc);
        resp_crates[n] = {
            username: username,
            name: name,
            desc: desc
        };
    });

    $(".upload").attr("action", server);
});

function get_crates(callback) {
    $.ajax({
        type: "GET",
        url: server,
        data: {action : "list"},
        crossDomain: true,
        success: function(resp) {
            console.log("Respond: " + resp);
            var crates = resp.split(",");
            for (var i = 0; i < crates.length; i++) {
                console.log("Requesting: " + i);
                $.ajax({
                    type: "GET",
                    url: server,
                    data: {action: "info", id: crates[i]},
                    crossDomain: true,
                    success: (function(n, id) {return function(data) {
                        console.log("Respond #" + n);
                        var resp = data.split(":");
                        callback(n, id, resp[0], resp[1], resp[2], resp[3], resp[4]);
                        if(data.indexOf("ERROR: ") == 0) {
                             $("#state").text(data);
                        }
                    }})(i, crates[i])
                });
            };
        }
    });
}


function upvote(crate_id) {
    $.ajax({
        type: "GET",
        url: server,
        data: {action: "vote", id: crate_id},
        crossDomain: true,
        success: function() {}
    });
}

function upload(username, reponame) {
    $.ajax({
        type: "GET",
        url: server,
        data: {host: "github", username: username, reponame: reponame},
        crossDomain: true,
        success: function(data) {
            if(data.indexOf("ERROR: ") == 0) {
                $("#state").value(data);
            }
        }
    });
}
