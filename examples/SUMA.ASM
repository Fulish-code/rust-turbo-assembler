;**********************************************************
; SUMA.ASM
;
; Este programa lee dos dígitos del teclado y despliega su
; suma (de 0 a 18). El pseudocódigo de este programa es:
;
;   BL = leeDigito()
;   BL += leeDigito()
;   despliega BL en decimal
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
pide1   db      'Primer digito: $'
pide2   db      13, 10, 'Segundo digito: $'
result  db      13, 10, 'La suma es: $'

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     dx, offset pide1        ; Pide el primer dígito
        mov     ah, 09h
        int     21h
        mov     ah, 01h                 ; Lee un carácter con eco
        int     21h
        sub     al, '0'                 ; ASCII -> número
        mov     bl, al                  ; BL = primer dígito

        mov     dx, offset pide2        ; Pide el segundo dígito
        mov     ah, 09h
        int     21h
        mov     ah, 01h
        int     21h
        sub     al, '0'
        add     bl, al                  ; BL += segundo dígito

        mov     dx, offset result
        mov     ah, 09h
        int     21h

        mov     al, bl                  ; AL = decenas,
        xor     ah, ah                  ; AH = unidades
        mov     cl, 10
        div     cl
        mov     bh, ah                  ; BH = unidades
        cmp     al, 0                   ; if(decenas == 0)
        je      unidad                  ;    goto unidad
        mov     dl, al                  ; Despliega las decenas
        add     dl, '0'
        mov     ah, 02h
        int     21h
unidad:
        mov     dl, bh                  ; Despliega las unidades
        add     dl, '0'
        mov     ah, 02h
        int     21h

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
