;**********************************************************
; SUM2DW_P.ASM
;
; Este programa suma dos variables de tipo palabra doble
; y guarda el resultado en una variable de tipo palabra
; doble. Este programa utiliza un procedimiento para
; efectuar la suma.
;
; El pseudocódigo de este programa es:
;
;    DX:AX = dato1
;    CX:BX = dato2
;
;    suma2dw()
;
;    resul = DX:AX
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
dato1   dd      ?
dato2   dd      ?
resul   dd      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     ax, [word dato1]        ; DX:AX = dato1
        mov     dx, [word dato1+2]
        mov     bx, [word dato2]        ; CX:BX = dato2
        mov     cx, [word dato2+2]

; Llama al procedimiento para efectuar la suma
        call    suma2dw                 ; suma2dw()

        mov     [word resul], ax        ; resul = DX:AX
        mov     [word resul+2], dx

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** PROCEDIMIENTOS ************************************

;**********************************************************
; SUMA2DW
;
; Este procedimiento suma dos variables de tipo
; palabra doble.
;
; Parámetros:
;
;       DX:AX = Primer sumando
;       CX:BX = Segundo sumando
;
; Regresa:
;
;       DX:AX = Suma
;
; El pseudocódigo de este procedimiento es:
;
;   DX:AX += CX:BX
;**********************************************************
proc    suma2dw
        add     ax, bx                  ; DX:AX += CX:BX
        adc     dx, cx

        ret                             ; Regresa del
                                        ; procedimiento
endp    suma2dw

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
