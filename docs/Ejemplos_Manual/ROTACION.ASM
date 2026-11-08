;**********************************************************
; ROTACION.ASM
;
; Este programa implementa todas las rotaciones con una
; variable de tipo palabra doble. El pseudocódigo de este
; programa es el siguiente:
;
;   DX:AX = dato
;   DX:AX = rcl DX:AX
;   r_rcl = DX:AX
;
;   DX:AX = dato
;   DX:AX = rcr DX:AX
;   r_rcr = DX:AX
;
;   DX:AX = dato
;   DX:AX = rol DX:AX
;   r_rol = DX:AX
;
;   DX:AX = dato
;   DX:AX = ror DX:AX
;   r_ror = DX:AX
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
dato    dd      ?
r_rcl   dd      ?
r_rcr   dd      ?
r_rol   dd      ?
r_ror   dd      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

; Rotación a la izquierda a través de la bandera de acarreo
        mov     ax, [word dato]         ; DX:AX = dato
        mov     dx, [word dato+2]

        rcl     ax, 1                   ; rcl DX:AX, 1
        rcl     dx, 1

        mov     [word r_rcl], ax        ; r_rcl = DX:AX
        mov     [word r_rcl+2], dx

; Rotación a la derecha a través de la bandera de acarreo
        mov     ax, [word dato]         ; DX:AX = dato
        mov     dx, [word dato+2]

        rcr     dx, 1                   ; rcr DX:AX, 1
        rcr     ax, 1

        mov     [word r_rcr], ax        ; r_rcr = DX:AX
        mov     [word r_rcr+2], dx

; Rotación a la izquierda
        mov     ax, [word dato]         ; DX:AX = dato
        mov     dx, [word dato+2]

        mov     bx, dx                  ; rol DX:AX, 1
        shl     bx, 1
        rcl     ax, 1
        rcl     dx, 1

        mov     [word r_rol], ax        ; r_rol = DX:AX
        mov     [word r_rol+2], dx

; Rotación a la derecha
        mov     ax, [word dato]         ; DX:AX = dato
        mov     dx, [word dato+2]

        mov     bx, ax                  ; ror DX:AX, 1
        shr     bx, 1
        rcr     dx, 1
        rcr     ax, 1

        mov     [word r_ror], ax        ; r_ror = DX:AX
        mov     [word r_ror+2], dx

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
