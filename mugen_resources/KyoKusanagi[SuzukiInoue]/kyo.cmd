

[Command]
name = "qcfqcf_x"
command = ~D, DF, F, D, DF, F, x
time = 30

[Command]
name = "qcfqcf_y"
command = ~D, DF, F, D, DF, F, y
time = 30

[Command]
name = "qcfqcf_a"
command = ~D, DF, F, D, DF, F, a
time = 30

[Command]
name = "qcfqcf_b"
command = ~D, DF, F, D, DF, F, b
time = 30

[Command]
name = "qcfqcf_ab"
command = ~D, DF, F, D, DF, F, a+b
time = 35

[Command]
name = "qcbhcf_xy"
command = D, DB, B, DB, D, DF, F, x+y
time = 40

[Command]
name = "qcbhcf_x"
command = D, DB, B, DB, D, DF, F, x
time = 40

[Command]
name = "qcbhcf_y"
command = D, DB, B, DB, D, DF, F, y
time = 40

[Command]
name = "qcfhcb_x"
command = D, DF, F, DF, D, DB, B, x
time = 40

[Command]
name = "qcfhcb_y"
command = D, DF, F, DF, D, DB, B, y
time = 40

[Command]
name = "qcf_x"
command = ~D, F, x
time = 15

[Command]
name = "qcf_y"
command = ~D, F, y
time = 15

[Command]
name = "qcb_x"
command = ~D, B, x
time = 15

[Command]
name = "qcb_y"
command = ~D, B, y
time = 15

[Command]
name = "hcb_x"
command = ~F, D, B, x
time = 30

[Command]
name = "hcb_y"
command = ~F, D, B, y
time = 30

[Command]
name = "hcb_a"
command = ~F, D, B, a
time = 30

[Command]
name = "hcb_b"
command = ~F, D, B, b
time = 30

[Command]
name = "rdp_a"
command = ~B, D, DB, a
time = 15

[Command]
name = "rdp_b"
command = ~B, D, DB, b
time = 15

[Command]
name = "qcf_a"
command = ~D, F, a
time = 15

[Command]
name = "qcf_b"
command = ~D, F, b
time = 15

[Command]
name = "dp_x"
command = ~F, D, DF, x
time = 15

[Command]
name = "dp_y"
command = ~F, D, DF, y
time = 15

[Command]
name = "charge_x"
command = ~30$B, F, x
time = 15

[Command]
name = "charge_y"
command = ~30$B, F, y
time = 15

[Command]
name = "charge_a"
command = ~30$B, F, a
time = 15

[Command]
name = "charge_b"
command = ~30$B, F, b
time = 15

[Command]
name = "longjump"
command = ~D, $U
time = 5

[Command]
name = "FF"
command = F, F
time = 10

[Command]
name = "BB"
command = B, B
time = 10

[Command]
name = "F,z"
command = /F,z
time = 1

[Command]
name = "recovery"
command = /F,x+a
time = 1

[Command]
name = "B,z"
command = /B,z
time = 1

[Command]
name = "dodge"
command = /B,x+a
time = 1

[Command]
name = "esc"
command = x+a
time = 1

[Command]
name = "knockdown"
command = y+b
time = 1

[Command]
name = "abc"
command = a+b+c
time = 1

[Command]
name = "counter"
command = x+y+a
time = 1

[Command]
name = "armor"
command = y+a+b
time = 1

[Command]
name = "throw1"
command = /F,b
time = 1

[Command]
name = "throw1"
command = /B,b
time = 1

[Command]
name = "throw2"
command = /F,y
time = 1

[Command]
name = "throw2"
command = /B,y
time = 1

[Command]
name = "fwd_a"
command = /F,a
time = 1

[Command]
name = "fwd_b"
command = /F,b

[Command]
name = "back_a"
command = /B,a

[Command]
name = "back_b"
command = /B,b

[Command]
name = "fwd_x"
command = /F,x

[Command]
name = "back_x"
command = /B,x

[Command]
name = "fwd_y"
command = /F,y

[Command]
name = "fwd_z"
command = /F,z
time = 1

[Command]
name = "back_z"
command = /B,z
time = 1

[Command]
name = "back_y"
command = /B,y

[Command]
name = "down_y"
command = /D, y

[Command]
name = "a"
command = a
time = 1

[Command]
name = "hold_a"
command = /$a
time = 1

[Command]
name = "b"
command = b
time = 1

[Command]
name = "hold_b"
command = /$b
time = 1

[Command]
name = "c"
command = c
time = 1

[Command]
name = "hold_c"
command = /c

[Command]
name = "x"
command = x
time = 1

[Command]
name = "hold_x"
command = /$x
time = 1

[Command]
name = "y"
command = y
time = 1

[Command]
name = "hold_y"
command = /$y
time = 1

[Command]
name = "z"
command = z
time = 1

[Command]
name = "hold_z"
command = /z

[Command]
name = "s"
command = s
time = 1

[Command]
name = "hold_s"
command = /s

[Command]
name = "holdfwd_x"
command = /$F, x
time = 1

[Command]
name = "holdfwd_y"
command = /$F, y
time = 1

[Command]
name = "holdfwd"
command = /$F
time = 1

[Command]
name = "holdback"
command = /$B
time = 1

[Command]
name = "holdup"
command = /$U
time = 1

[Command]
name = "holddown"
command = /$D
time = 1

[Command]
name = "holdDF_b"
command = /DF,b
time = 1






[Statedef -1]
[State -1,-1]
type = ChangeState
value = 3700
triggerall = command = "qcfqcf_ab"
triggerall = power >= 3000
triggerall = statetype != A
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,0]
type = ChangeState
value = 3600
triggerall = (command = "qcfqcf_a" || command = "qcfqcf_b")
triggerall = power >= 2000
triggerall = statetype != A
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,1]
type = ChangeState
value = 3500
triggerall = command = "qcfhcb_y"
triggerall = power >= 2000
triggerall = statetype != A
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,2]
type = ChangeState
value = 3400
triggerall = command = "qcfhcb_x"
triggerall = power >= 1000
triggerall = statetype != A
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,3]
type = ChangeState
value = 3300
triggerall = command = "qcbhcf_xy"
triggerall = power >= 3000
triggerall = statetype != A
triggerall =  PalNo = [1,5]
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,4]
type = ChangeState
value = 3350
triggerall = (command = "qcbhcf_x" || command = "qcbhcf_y")
triggerall = (power >= 1000 && life <= 200)
triggerall = statetype != A
triggerall =  PalNo = 6
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1


[State -1,5]
type = ChangeState
value = 3200
triggerall = (command = "qcbhcf_x" || command = "qcbhcf_y")
triggerall = power >= 1000
triggerall = statetype != A
triggerall =  PalNo = [1,5]
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,6]
type = ChangeState
value = 3250
triggerall = (command = "qcbhcf_x" || command = "qcbhcf_y")
triggerall = power >= 1000
triggerall = statetype != A
triggerall =  PalNo = 6
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1


[State -1,7]
type = ChangeState
value = 3100
triggerall = command = "qcfqcf_y"
triggerall = power >=2000
triggerall = statetype != A
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,8]
type = ChangeState
value = 3150
triggerall = (command = "qcfqcf_x" || command = "qcfqcf_y")
triggerall = power >=1000
triggerall = statetype != A
triggerall = PalNo = 3
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,9]
type = ChangeState
value = 3080
triggerall = (command = "qcfqcf_x" || command = "qcfqcf_y")
triggerall = power >= 1000
triggerall = life <= 200
triggerall = statetype != A
triggerall = PalNo = 6
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1


[State -1,10]
type = ChangeState
value = 3000
triggerall = command = "qcfqcf_x"
triggerall = power >= 1000
triggerall = statetype != A
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1

[State -1,11]
type = ChangeState
value = 3050
triggerall = (command = "qcfqcf_x" || command = "qcfqcf_y")
triggerall = power >= 1000
triggerall = statetype != A
triggerall = PalNo = 6
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1


[State -1,12]
type = ChangeState
value = 1500
triggerall = command = "dp_x"
trigger1 = statetype != A
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1



[State -1,13]
type = ChangeState
value = 1520
triggerall = command = "dp_y"
trigger1 = statetype != A
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,14]
type = ChangeState
value = 1000
triggerall = command = "qcf_x"
trigger1 = statetype != A
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1



[State -1,15]
type = VarSet
v = 1
value = 1
triggerall = stateno = 1000
triggerall = time > 2
trigger1 = command = "qcf_x"
trigger2 = command = "qcf_y"


[State -1,16]
type = VarSet
v = 1
value = 1
triggerall = stateno = 1005
triggerall = time > 2
trigger1 = command = "x"
trigger2 = command = "y"


[State -1,17]
type = VarSet
v = 1
value = 2
triggerall = stateno = 1000
triggerall = time > 2
trigger1 = command = "hcb_x"
trigger2 = command = "hcb_y"


[State -1,18]
type = VarSet
v = 1
value = 1
triggerall = stateno = 1020
triggerall = time > 2
trigger1 = command = "x"
trigger2 = command = "y"


[State -1,19]
type = VarSet
v = 1
value = 2
triggerall = time > 2
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = stateno = 1005
trigger1 = command = "a"
trigger2 = stateno = 1020
trigger2 = command = "a"


[State -1,20]
type = VarSet
v = 1
value = 2
triggerall = time > 2
triggerall = ( PalNo = 3 || PalNo = 6 )
trigger1 = stateno = 1005
trigger1 = ( command = "a" || command = "b" )
trigger2 = stateno = 1020
trigger2 = ( command = "a" || command = "b" )



[State -1,21]
type = VarSet
v = 1
value = 3
triggerall = time > 2
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = stateno = 1005
trigger1 = command = "b"
trigger2 = stateno = 1020
trigger2 = command = "b"


[State -1,22]
type = ChangeState
value = 1300
triggerall = command = "qcf_y"
trigger1 = statetype != A
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,23]
type = VarSet
v = 1
value = 1
triggerall = stateno = 1300
trigger1 = command = "qcb_x"
trigger2 = command = "qcb_y"


[State -1,24]
type = VarSet
v = 1
value = 1
triggerall = stateno = 1305
trigger1 = command = "holdfwd_x"
trigger2 = command = "holdfwd_y"


[State -1,25]
type = VarSet
v = 1
value = 1
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
triggerall = stateno = 1312
trigger1 = command = "dp_x"
trigger2 = command = "dp_y"


[State -1,26]
type = ChangeState
value = 1100
triggerall = command = "qcb_x"
trigger1 = statetype != A
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,27]
type = ChangeState
value = 1105
triggerall = command = "qcb_y"
trigger1 = statetype != A
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,28]
type = ChangeState
value = 1200
triggerall = command = "hcb_a"
triggerall = statetype != A
triggerall =  PalNo = [1,5]
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,29]
type = ChangeState
value = 1230
triggerall = command = "hcb_b"
triggerall = statetype != A
triggerall =  PalNo = [1,5]
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1



[State -1,30]
type = ChangeState
value = 1240
triggerall = command = "charge_a"
triggerall = statetype != A
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,31]
type = ChangeState
value = 1240
triggerall = command = "hcb_a"
triggerall = statetype != A
triggerall =  PalNo = 6
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1



[State -1,32]
type = ChangeState
value = 1250
triggerall = command = "charge_b"
triggerall = statetype != A
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1


[State -1,33]
type = ChangeState
value = 1250
triggerall = command = "hcb_b"
triggerall = statetype != A
triggerall =  PalNo = 6
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1


[State -1,34]
type = ChangeState
value = 1400
triggerall = command = "rdp_a"
trigger1 = statetype != A
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,35]
type = ChangeState
value = 1450
triggerall = command = "rdp_b"
trigger1 = statetype != A
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,36]
type = ChangeState
value = 1600
triggerall = command = "qcf_a"
trigger1 = statetype != A
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,37]
type = ChangeState
value = 1610
triggerall = command = "qcf_b"
trigger1 = statetype != A
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,38]
type = ChangeState
value = 1700
triggerall = command = "charge_x"
triggerall = numprojID(100) = 0
triggerall = statetype != A
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,39]
type = ChangeState
value = 1701
triggerall = command = "charge_y"
triggerall = numprojID(100) = 0
triggerall = statetype != A
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = ctrl = 1
trigger2 = stateno = 200
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = movecontact = 1
trigger5 = stateno = 218
trigger5 = movecontact = 1
trigger6 = stateno = 221
trigger6 = movecontact = 1
trigger7 = stateno = 231
trigger7 = movecontact = 1
trigger8 = stateno = 400
trigger8 = movecontact = 1
trigger9 = stateno = 410
trigger9 = movecontact = 1
trigger10 = stateno = 420
trigger10 = movecontact = 1
trigger11 = stateno = 430
trigger11 = movecontact = 1
trigger12 = stateno = 240
trigger12 = movecontact = 1
trigger13 = stateno = 235
trigger13 = movecontact = 1


[State -1,40]
type = ChangeState
value = 435
triggerall = command = "holdDF_b"
trigger1 = (( ctrl && statetype = C ) || stateno = 100 )
trigger2 = stateno = 200
trigger2 = command = "holddown"
trigger2 = movecontact = 1
trigger3 = stateno = 201
trigger3 = command = "holddown"
trigger3 = movecontact = 1
trigger4 = stateno = 211
trigger4 = command = "holddown"
trigger4 = movecontact = 1
trigger5 = stateno = 221
trigger5 = command = "holddown"
trigger5 = movecontact = 1
trigger6 = stateno = 231
trigger6 = command = "holddown"
trigger6 = movecontact = 1
trigger7 = stateno = 400
trigger7 = command = "holddown"
trigger7 = movecontact = 1
trigger8 = stateno = 410
trigger8 = command = "holddown"
trigger8 = movecontact = 1
trigger9 = stateno = 420
trigger9 = command = "holddown"
trigger9 = movecontact = 1
trigger10 = stateno = 430
trigger10 = command = "holddown"
trigger10 = movecontact = 1


[State -1,41]
type = ChangeState
value = 640
triggerall = statetype = A
triggerall = ctrl = 1
trigger1 = command = "knockdown"
trigger2 = command = "c"


[State -1,42]
type = ChangeState
value = 600
trigger1 = command = "x"
trigger1 = statetype = A
trigger1 = ctrl = 1


[State -1,43]
type = ChangeState
value = 615
trigger1 = command = "a"
trigger1 = statetype = A
trigger1 = Vel X != 0
trigger1 = ctrl = 1


[State -1,44]
type = ChangeState
value = 610
trigger1 = command = "a"
trigger1 = statetype = A
trigger1 = ctrl = 1


[State -1,45]
type = ChangeState
value = 625
triggerall =( command = "down_y" && statetype = A )
trigger1 = ctrl = 1
trigger2 = stateno = 106


[State -1,46]
type = ChangeState
value = 620
trigger1 = command = "y"
trigger1 = statetype = A
trigger1 = ctrl = 1


[State -1,47]
type = ChangeState
value = 635
trigger1 = command = "b"
trigger1 = statetype = A
trigger1 = Vel X != 0
trigger1 = ctrl = 1


[State -1,48]
type = ChangeState
value = 630
trigger1 = command = "b"
trigger1 = statetype = A
trigger1 = ctrl = 1


[State -1,49]
type = ChangeState
value = 400
triggerall = ( command = "x" && command = "holddown" )
trigger1 = ( ctrl = 1 && statetype = C )
trigger2 = ( Time > 5 && stateno = 400 && movecontact = 0 )
trigger3 = ( Time > 7 && stateno = 400 && movecontact = 1 )
trigger4 = ( stateno = 410 && movecontact = 1 )
trigger5 = ( Time > 5 && stateno = 100 )


[State -1,50]
type = ChangeState
value = 410
triggerall = ( command = "a" && command = "holddown" )
trigger1 = ( ctrl = 1 && statetype = C )
trigger2 = ( Time > 3 && stateno = 410 && movecontact = 0 )
trigger3 = ( Time > 5 && stateno = 410 && movecontact = 1 )
trigger4 = ( stateno = 400 && movecontact = 1 )
trigger5 = ( Time > 8 && stateno = 211 )
trigger6 = ( Time > 5 && stateno = 100 )


[State -1,51]
type = ChangeState
value = 420
triggerall = ( command = "y" && command = "holddown" )
trigger1 = ( ctrl = 1 && statetype = C )
trigger2 = ( Time > 5 && stateno = 100 )


[State -1,52]
type = ChangeState
value = 430
triggerall = ( command = "b" && command = "holddown" && statetype = C )
trigger1 = ctrl = 1
trigger2 = ( Time > 5 && stateno = 100 )


[State -1,53]
type = ChangeState
value = 510
trigger1 = command = "throw2"
trigger1 = statetype = S
trigger1 = stateno != 100
trigger1 = p2bodydist x <= 7
trigger1 = p2movetype != H
trigger1 = p2statetype != A
trigger1 = ctrl = 1


[State -1,54]
type = ChangeState
value = 500
trigger1 = command = "throw1"
trigger1 = statetype = S
trigger1 = stateno != 100
trigger1 = p2bodydist x <= 7
trigger1 = p2movetype != H
trigger1 = p2statetype != A
trigger1 = ctrl = 1


[State -1,55]
type = ChangeState
value = 260
trigger1 = command = "dodge" ^^ command = "z"
trigger1 = command = "holdback" && statetype = S && ctrl


[State -1,56]
type = ChangeState
value = 250
trigger1 = command = "recovery" ^^ command = "z"
trigger1 = command = "holdfwd" && statetype = S && ctrl

escape

[State -1,57]
type =ChangeState
value = 280
trigger1 = command = "esc"
trigger1 = statetype != A
trigger1 = ctrl
trigger2 = command = "z"
trigger2 = statetype != A
trigger2 = ctrl


[State -1,58]
type = ChangeState
value = 255
triggerall = stateno >= 150
triggerall = stateno <= 151
triggerall = command = "holdback"
triggerall = power >= 1000
trigger1 = command = "recovery"
trigger2 = command = "fwd_z"


[State -1,59]
type = ChangeState
value = 265
triggerall = stateno >= 150
triggerall = stateno <= 151
triggerall = command = "holdback"
triggerall = power >= 1000
trigger1 = command = "dodge"
trigger2 = command = "back_z"


[State -1,60]
type = ChangeState
value = 270
triggerall = power >= 1000
triggerall = stateno >= 150
triggerall = stateno <= 151
trigger1 = command = "knockdown"
trigger2 = command = "c"


[State -1,61]
type = ChangeState
value = 240
triggerall = statetype != A
triggerall = ctrl = 1
trigger1 = command = "knockdown"
trigger2 = command = "c"


[State -1,62]
type = ChangeState
value = 290
triggerall = ( stateno = 280 && time = [5,20] )
trigger1 = ( command = "x" || command = "y" || command = "a" || command = "b" )


[State -1,63]
type = ChangeState
value = 235
triggerall = ( command = "back_a" && command != "holddown" )
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 )
trigger1 = ( ctrl = 1 && statetype = S )
trigger2 = ( Time > 5 && stateno = 100 )
trigger3 = ( stateno = 201 || stateno = 220 || stateno = 221 )


[State -1,64]
type = ChangeState
value = 218
trigger1 = ( command = "fwd_a" && movecontact )
trigger1 = ( stateno = 201 || stateno = 211 || stateno = 220 || stateno = 221 || stateno = 231 || stateno = 400 || stateno = 420 || stateno = 430 )


[State -1,65]
type = ChangeState
value = 215
triggerall = command = "fwd_a"
triggerall = command != "holddown"
trigger1 = statetype = S
trigger1 = ctrl = 1


[State -1,66]
type = ChangeState
value = 200
triggerall = ( command = "x" && command != "holddown" )
triggerall = P2BodyDist X > 15
trigger1 = ( ctrl = 1 && statetype = S )
trigger2 = ( stateno = 211 && movecontact = 1 )
trigger3 = ( Time > 5 && stateno = 410 && movecontact = 1 )
trigger4 = ( Time > 5 && stateno = 100 )
trigger5 = ( Time > 7 && stateno = 201 && movecontact = 1 )


[State -1,67]
type = ChangeState
value = 201
triggerall = ( command = "x" && command != "holddown" )
triggerall = P2BodyDist X <= 15
trigger1 = ( ctrl = 1 && statetype = S  )
trigger2 = ( stateno = 211 && movecontact = 1 )
trigger3 = ( Time > 6 && stateno = 201 && movecontact = 1 )
trigger4 = ( Time > 5 && stateno = 410 && movecontact = 1 )
trigger5 = ( Time > 5 && stateno = 100 )


[State -1,68]
type = ChangeState
value = 210
triggerall = ( command = "a" && command != "holddown" )
triggerall = P2bodydist X > 20
trigger1 = ( ctrl = 1 && statetype = S )
trigger2 = ( Time > 8 && stateno = 211 )
trigger3 = ( Time > 5 && stateno = 410 )
trigger4 = ( Time > 5 && stateno = 100 )


[State -1,69]
type = ChangeState
value = 211
triggerall = ( command = "a" && command != "holddown" )
triggerall = P2bodydist X <= 20
trigger1 = ( ctrl = 1 && statetype = S )
trigger2 = ( Time > 7 && stateno = 211 && movecontact = 1 )
trigger3 = ( Time > 8 && stateno = 211 )
trigger4 = ( Time > 5 && stateno = 410 )
trigger5 = ( Time > 5 && stateno = 100 )


[State -1,70]
type = ChangeState
value = 220
triggerall = ( command = "y" && command != "holddown" )
triggerall = P2BodyDist X > 25
trigger1 = ( ctrl = 1 && statetype = S )
trigger2 = ( Time > 5 && stateno = 100 )
trigger3 = ( Time > 9 && stateno = 410 && movecontact = 1 )


[State -1,71]
type = ChangeState
value = 221
triggerall = ( command = "y" && command != "holddown" )
triggerall = P2BodyDist X <= 25
trigger1 = ( ctrl = 1 && statetype = S )
trigger2 = ( Time > 4 && stateno = 201 )
trigger3 = ( Time > 4 && stateno = 211 )
trigger4 = ( Time > 5 && stateno = 100 )
trigger5 = ( Time > 11 && stateno = 410 )


[State -1,72]
type = ChangeState
value = 230
triggerall = ( command = "b" && command != "holddown" )
triggerall = ( PalNo = 1 || PalNo = 2 || PalNo = 4 || PalNo = 5 || PalNo = 6 )
triggerall = P2BodyDist X > 27
trigger1 = ( ctrl = 1 && statetype = S )
trigger2 = ( Time > 5 && stateno = 100 )


[State -1,73]
type = ChangeState
value = 235
triggerall = ( command = "b" && command != "holddown" )
triggerall =  PalNo = 3
triggerall = P2BodyDist X > 27
trigger1 = ( ctrl = 1 && statetype = S )
trigger2 = ( Time > 5 && stateno = 100 )


[State -1,74]
type = ChangeState
value = 231
triggerall = ( command = "b" && command != "holddown" )
triggerall = P2BodyDist X <= 27
trigger1 = ( ctrl = 1 && statetype = S )
trigger2 = ( Time > 5 && stateno = 100 )


[State -1,75]
type = ChangeState
value = 195
trigger1 = command = "s"
trigger1 = command != "holddown"
trigger1 = statetype = S
trigger1 = ctrl = 1


[State -1,76]
type = ChangeState
value = 100
trigger1 = command = "FF"
trigger1 = statetype = S
trigger1 = command != "holddown"
trigger1 = ctrl = 1


[State -1,77]
type = ChangeState
value = 105
trigger1 = command = "BB"
trigger1 = statetype = S
trigger1 = command != "holddown"
trigger1 = ctrl = 1



