#!/bin/bash
systemctl stop nclogin.service
SCRIPTPATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
cp $SCRIPTPATH/../target/release/loginrs /sbin/loginrs
systemctl start nclogin.service
