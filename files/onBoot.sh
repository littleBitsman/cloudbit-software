#!/bin/bash

# onBoot.sh: runs at boot by systemctl after required daemons.
# reads from ROM every time and writes hash and mac to temporary files
# also decides whether to do testing or enter "normal" mode.

# include global variables
source /etc/lb_scripts.conf

sudo chmod 777 /usr/local/lb/cloud_client/bin/cloud_client

systemctl is-active cloudclient --quiet
if (( ! $? )); then
	systemctl stop cloudclient
fi

printf "Checking cloud identity on ROM... "

# read from ROM
hash=$( $TESTPATH/OTPread.sh )
emptyROM=00000000000000000000000000000000
# if nothing is stored in ROM, then run write_cloudID
if [[ $hash == $emptyROM ]]; then
	printf "Identity missing, writing to ROM... "

	#write to rom
	eval $LEDPATH/setColor teal
	hash=$( eval $TESTPATH/writeCloudID.sh )
	status=$?
	eval $LEDPATH/setColor off
	
	# make sure VDDIO is set back to 3.3V
	vddioVal=0x1014
	outVal=$(devmem2 $(( 0x80044000 + 0x060 )))
	if [[ ${outVal:(-4)} -ne 1014 ]]; then
		newVal=$(devmem2 $(( 0x80044000 + 0x060 )) w 0x1014 | awk 'NR==2 {print $0}')
	fi

	# make sure clock is set back to PLL
	clkset=$(devmem2 $(( 0x80040000 + 0x118 )))
	if [[ $clkset -ne 0x11A ]]; then
		readback=$(devmem2 $(( 0x80040000 + 0x118 )) w 0x80 | awk 'NR==2 {print $0}')
	fi

	if [[ $status != 0 ]]; then
		echo "ERROR: Unable to create cloud identity." >&2

		# blink red endlessly
		eval $LEDPATH/setColor blink red
		exit 1
	fi

	# write identity and mac to files
	echo $hash > $CLOUDID
	eval $TESTPATH/writeMAC.sh


	echo "CREATED"
	
else
	echo "EXISTS"
	
	# write identity and mac to file
	echo $hash > $CLOUDID
	eval $TESTPATH/writeMAC.sh
	if (( $? )); then
		echo "ERROR: Unable to create cloud identity." >&2

		# blink red endlessly
		eval $LEDPATH/setColor hold red

		exit 1
	fi
	
fi

# check for salt and delete if it still exists
if [ -f $SALT ]; then
	printf "Securely erasing helper files... "
	shred -u -z -n 8 $SALT
	echo "OK"
fi

# check test results
printf "Checking cloudbit test results... "

# if tests are run in fullTest, /var/lb/runTestStatus won't be empty
eval $TESTPATH/fullTest.sh > $TMP_DIR/runTestStatus
eval $TESTPATH/getTestStatus.sh
status=$?
if [[ $status != 0 ]] ; then
	# at least one test failed or board hasn't been tested.
	# blink red endlessly and halt system
	eval $LEDPATH/setColor blink red
	sleep 3
	systemctl halt
fi

# pretty sure this literally cannot happen
# - littleBitsman

# check if /var/lb/runTestStatus isn't empty
# this implies tests were run - set LED green if they passed then halt system
if [ -s $TMP_DIR/runTestStatus ]; then
	if [[ $status == 0 ]]; then
		eval $LEDPATH/setColor blink teal
	fi
	sleep 3
	eval $LEDPATH/setColor hold green
	systemctl halt
fi

# restart DAC
systemctl restart dac

exit 0
