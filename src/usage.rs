static USAGE_TEXT:&'static str = "
usage:
    Starting the port mapper daemon:
       epmd [-d|-debug] [DbgExtra...] [-address List]
            [-port No] [-daemon] [-relaxed_command_check]

    Communicating with a running port mapper daemon:
       epmd [-d|-debug]
       epmd [-port <number>]
       epmd [-names|-kill|-stop name]

See the Erlang epmd manual page for info about the usage.

Regular options
    -address <list>
        Let epmd listen only on the comma-separated list of IP
        addresses (and on the loopback interface)
    -port <number>
        Let epmd listen to another port than default %d
    -d
    -debug
        Enable debugging. This will give a log to
        the standard error stream. It will shorten
        the number of saved used node names to 5.
        If you give more than one debug flag you may
        get more debugging information.
    -daemon
        Start epmd detached (as a daemon)
    -relaxed_command_check
        Allow this instance of epmd to be killed with
        epmd -kill even if there nodes.
        Also allows forced unregister (epmd -stop).

DbgExtra options
    -packet_timeout <seconds>
        Set the number of seconds a connection can be
        inactive before epmd times out and closes the
        connection (default 60).
    -delay_accept <seconds>
        To simulate a busy server you can insert a
        delay between epmd gets notified about that
        a new connection is requested and when the
        connections gets accepted.
    -delay_write <seconds>
        Also a simulation of a busy server. Inserts
        a delay before a reply is sent.

Interactive options
    -names
        List names registered with the currently
    -kill
        Kill the currently running epmd
        (only allowed if -names show empty database or
        -relaxed_command_check was given when epmd was started).
    -stop Name
        Forcibly unregisters a name with epmd
        (only allowed if -relaxed_command_check was given when
        epmd was started).
    -systemd (if available)
        Wait for socket from systemd. The option makes sense
        when started from .socket unit.
";

pub fn display_usage () {
    println!("{}", USAGE_TEXT);
}
